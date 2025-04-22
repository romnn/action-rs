pub mod env;
pub mod input;
pub mod summary;
pub mod utils;

use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "derive")]
pub use action_derive::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum LogLevel {
    Debug,
    Error,
    Warning,
    Notice,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warning => write!(f, "warning"),
            LogLevel::Notice => write!(f, "notice"),
        }
    }
}

/// Prepare key value message.
///
/// # Errors
/// If the value contains the randomly generated delimiter.
pub fn prepare_kv_message(key: &str, value: &str) -> Result<String, ValueError> {
    use uuid::Uuid;
    let delimiter = format!("ghadelimiter_{}", Uuid::new_v4());

    // These should realistically never happen,
    // but just in case someone finds a way to exploit
    // uuid generation let's not allow keys or values that
    // contain the delimiter.
    if key.contains(&delimiter) {
        return Err(ValueError::ContainsDelimiter { delimiter });
    }

    if value.contains(&delimiter) {
        return Err(ValueError::ContainsDelimiter { delimiter });
    }
    Ok(format!("{key}<<{delimiter}\n{value}\n{delimiter}"))
}

/// Sets env variable for this action and future actions in the job.
///
/// # Errors
/// If the file command fails.
pub fn export_var(
    env: &(impl env::Read + env::Write),
    name: impl AsRef<str>,
    value: impl Into<String>,
) -> Result<(), CommandError> {
    let value = value.into();
    env.set(name.as_ref(), &value);

    if env.get("GITHUB_ENV").is_some() {
        let message = prepare_kv_message(name.as_ref(), &value)?;
        issue_file_command("ENV", message)?;
        return Ok(());
    }

    issue(
        &CommandBuilder::new("set-env", value)
            .property("name", name.as_ref())
            .build(),
    );
    Ok(())
}

/// Registers a secret which will get masked from logs.
pub fn set_secret(secret: impl Into<String>) {
    issue(&CommandBuilder::new("add-mask", secret).build());
}

/// Prepends a path to the `PATH` environment variable.
///
/// # Errors
/// If the paths can not be joined.
fn prepend_to_path(
    env: &impl env::Write,
    path: impl AsRef<Path>,
) -> Result<(), std::env::JoinPathsError> {
    if let Some(old_path) = std::env::var_os("PATH") {
        let paths = [path.as_ref().to_path_buf()]
            .into_iter()
            .chain(std::env::split_paths(&old_path));
        let new_path = std::env::join_paths(paths)?;
        env.set("PATH", new_path);
    }
    Ok(())
}

pub trait Parse {
    type Input;

    #[must_use]
    fn parse() -> HashMap<Self::Input, Option<String>> {
        Self::parse_from(&env::OsEnv)
    }

    #[must_use]
    fn parse_from<E: env::Read>(env: &E) -> HashMap<Self::Input, Option<String>>;
}

/// Enables or disables the echoing of commands into stdout for the rest of the step.
///
/// Echoing is disabled by default if `ACTIONS_STEP_DEBUG` is not set.
pub fn set_command_echo(enabled: bool) {
    issue(&CommandBuilder::new("echo", if enabled { "on" } else { "off" }).build());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum ExitCode {
    /// A code indicating that the action was successful
    Success = 0,
    /// A code indicating that the action was a failure
    Failure = 1,
}

/// Sets the action status to failed.
///
/// When the action exits it will be with an exit code of 1.
pub fn fail(message: impl std::fmt::Display) {
    error!("{}", message);
    std::process::exit(ExitCode::Failure as i32);
}

/// Gets whether Actions Step Debug is on or not.
#[must_use]
pub fn is_debug() -> bool {
    std::env::var("RUNNER_DEBUG")
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
}

#[derive(Debug)]
pub struct CommandBuilder {
    command: String,
    message: String,
    props: HashMap<String, String>,
}

impl CommandBuilder {
    #[must_use]
    pub fn new(command: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            message: message.into(),
            props: HashMap::new(),
        }
    }

    #[must_use]
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    #[must_use]
    pub fn properties(mut self, props: HashMap<String, String>) -> Self {
        self.props.extend(props);
        self
    }

    #[must_use]
    pub fn build(self) -> Command {
        let Self {
            command,
            message,
            props,
        } = self;
        Command {
            command,
            message,
            props,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Command {
    command: String,
    message: String,
    props: HashMap<String, String>,
}

impl Command {
    #[must_use]
    pub fn new(command: String, message: String, props: HashMap<String, String>) -> Self {
        Self {
            command,
            message,
            props,
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const CMD_STRING: &str = "::";
        write!(f, "{}{}", CMD_STRING, self.command)?;
        if !self.props.is_empty() {
            write!(f, " ")?;
        }
        for (i, (k, v)) in self.props.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            if v.is_empty() {
                continue;
            }
            write!(f, "{k}={}", utils::escape_property(v))?;
        }
        write!(f, "{}{}", CMD_STRING, self.message)
    }
}

pub fn issue(cmd: &Command) {
    println!("{cmd}");
}

#[derive(thiserror::Error, Debug)]
pub enum ValueError {
    #[error("should not contain delimiter `{delimiter}`")]
    ContainsDelimiter { delimiter: String },
}

#[derive(thiserror::Error, Debug)]
pub enum FileCommandError {
    #[error("missing env variable for file command {cmd}")]
    Missing {
        source: std::env::VarError,
        cmd: String,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Value(#[from] ValueError),
}

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error(transparent)]
    File(#[from] FileCommandError),

    #[error(transparent)]
    Value(#[from] ValueError),
}

/// Issue a file command.
///
/// # Errors
/// When no env variable for the file command exists or writing fails.
pub fn issue_file_command(
    command: impl AsRef<str>,
    message: impl AsRef<str>,
) -> Result<(), FileCommandError> {
    use std::io::Write;
    let key = format!("GITHUB_{}", command.as_ref());
    let file_path = std::env::var(key).map_err(|source| FileCommandError::Missing {
        source,
        cmd: command.as_ref().to_string(),
    })?;
    let file = std::fs::OpenOptions::new().append(true).open(file_path)?;
    let mut file = std::io::BufWriter::new(file);
    writeln!(file, "{}", message.as_ref())?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum AddPathError {
    #[error(transparent)]
    File(#[from] FileCommandError),

    #[error(transparent)]
    Join(#[from] std::env::JoinPathsError),
}

/// Prepends a path to the `PATH` environment variable.
///
/// Persisted for this action and future actions.
///
/// # Errors
/// If the file command
pub fn add_path(
    env: &(impl env::Read + env::Write),
    path: impl AsRef<Path>,
) -> Result<(), AddPathError> {
    let path_string = path.as_ref().to_string_lossy();
    prepend_to_path(env, path.as_ref())?;

    if env.get("GITHUB_PATH").is_some() {
        issue_file_command("PATH", &path_string)?;
    } else {
        issue(&CommandBuilder::new("add-path", path_string).build());
    }
    Ok(())
}

// pub fn issue_command(
//     command: impl AsRef<str>,
//     message: impl std::fmt::Display,
//     props: HashMap<String, String>,
// ) {
//     let cmd= Command::new(command.as_ref(), message.to_string(), props);
//     issue();
// }

#[derive(Default, Debug, Hash, PartialEq, Eq)]
pub struct AnnotationProperties {
    pub title: Option<String>,
    pub file: Option<String>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub start_column: Option<usize>,
    pub end_column: Option<usize>,
}

impl<H> From<AnnotationProperties> for HashMap<String, String, H>
where
    H: std::hash::BuildHasher + Default,
{
    fn from(props: AnnotationProperties) -> Self {
        [
            ("title".to_string(), props.title),
            ("file".to_string(), props.file),
            (
                "line".to_string(),
                props.start_line.map(|line| line.to_string()),
            ),
            (
                "endLine".to_string(),
                props.end_line.map(|line| line.to_string()),
            ),
            (
                "col".to_string(),
                props.start_column.map(|col| col.to_string()),
            ),
            (
                "endColumn".to_string(),
                props.end_column.map(|col| col.to_string()),
            ),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
        .collect()
    }
}

/// Adds an error issue.
pub fn issue_level(
    level: LogLevel,
    message: impl Into<String>,
    props: Option<AnnotationProperties>,
) {
    let props = props.unwrap_or_default();
    issue(
        &CommandBuilder::new(level.to_string(), message)
            .properties(props.into())
            .build(),
    );
}

// /// Writes debug message to user log.
// pub fn debug(message: impl std::fmt::Display) {
//     issue_command("debug", message, HashMap::new())
// }

// /// Adds an error issue.
// pub fn error(message: impl ToString, props: AnnotationProperties) {
//     issue_level(LogLevel::Error, message, props);
// }

#[macro_export]
macro_rules! debug {
        ($($arg:tt)*) => {{
            $crate::issue_level(
                $crate::LogLevel::Debug,
                format!($($arg)*),
                None,
            );
        }};
    }

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        $crate::issue_level(
            $crate::LogLevel::Warning,
            format!($($arg)*),
            None,
        );
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        $crate::issue_level(
            $crate::LogLevel::Error,
            format!($($arg)*),
            None,
        );
    }};
}

#[macro_export]
macro_rules! notice {
    ($($arg:tt)*) => {{
        $crate::issue_level(
            $crate::LogLevel::Notice,
            format!($($arg)*),
            None,
        );
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => { println!($($arg)*); };
}

// /// Adds a warning issue.
// pub fn issue_warning(message: impl ToString, props: AnnotationProperties) {
//     issue_level(LogLevel::Warning, message, props);
// }
//
// /// Adds a notice issue
// pub fn notice(message: impl std::fmt::Display, props: AnnotationProperties) {
//     issue_level(LogLevel::Notice, message, props);
// }

/// Begin an output group.
///
/// Output until the next `group_end` will be foldable in this group.
pub fn start_group(name: impl Into<String>) {
    issue(&CommandBuilder::new("group", name).build());
}

/// End an output group.
pub fn end_group() {
    issue(&CommandBuilder::new("endgroup", "").build());
}

/// Saves state for current action, the state can only be retrieved by this action's post job execution.
///
/// # Errors
/// If the file command fails.
pub fn save_state(
    env: &impl env::Read,
    name: impl AsRef<str>,
    value: impl Into<String>,
) -> Result<(), CommandError> {
    if env.get("GITHUB_STATE").is_some() {
        let message = prepare_kv_message(name.as_ref(), &value.into())?;
        issue_file_command("STATE", message)?;
        return Ok(());
    }

    issue(
        &CommandBuilder::new("save-state", value)
            .property("name", name.as_ref())
            .build(),
    );
    Ok(())
}

/// Gets the value of an state set by this action's main execution.
#[must_use]
pub fn get_state(name: impl AsRef<str>) -> Option<String> {
    std::env::var(format!("STATE_{}", name.as_ref())).ok()
}

/// Wrap an asynchronous function call in a group.
///
/// Returns the same type as the function itself.
pub async fn group<T>(name: impl Into<String>, fut: impl std::future::Future<Output = T>) -> T {
    start_group(name);
    let res: T = fut.await;

    end_group();
    res
}
