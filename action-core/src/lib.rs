// #![allow(warnings)]

use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "derive")]
pub use action_derive::Action;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Input<'a> {
    pub description: Option<&'a str>,
    pub deprecation_message: Option<&'a str>,
    pub default: Option<&'a str>,
    pub required: Option<bool>,
}

#[derive(Debug)]
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

pub fn input_env_var(name: impl Into<String>) -> String {
    let mut var: String = name.into();
    if !var.starts_with("INPUT_") {
        var = format!("INPUT_{}", var);
    }
    var = var.replace(' ', "_").to_uppercase();
    var
}

pub mod env {
    use std::collections::HashMap;

    #[derive(Debug, Default)]
    pub struct Env(pub HashMap<String, String>);

    impl FromIterator<(String, String)> for Env {
        fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
            Self::new(HashMap::from_iter(iter))
        }
    }

    impl Env {
        pub fn new(values: HashMap<String, String>) -> Self {
            let inner = values
                .into_iter()
                .map(|(k, v)| (super::input_env_var(k), v))
                .collect();
            Self(inner)
        }

        #[cfg(feature = "serde")]
        pub fn from_str(env: &str) -> Result<Self, serde_yaml::Error> {
            Ok(Self::new(serde_yaml::from_str(env)?))
        }

        #[cfg(feature = "serde")]
        pub fn from_reader(reader: impl std::io::Read) -> Result<Self, serde_yaml::Error> {
            Ok(Self::new(serde_yaml::from_reader(reader)?))
        }
    }

    impl std::borrow::Borrow<HashMap<String, String>> for Env {
        fn borrow(&self) -> &HashMap<String, String> {
            &self.0
        }
    }

    impl std::ops::Deref for Env {
        type Target = HashMap<String, String>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::ops::DerefMut for Env {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    pub trait ReadEnv {
        fn get(&self, key: &str) -> Result<String, std::env::VarError>;
    }

    pub trait WriteEnv {
        fn set(&mut self, key: String, value: String);
    }

    impl<T> ReadEnv for T
    where
        T: std::borrow::Borrow<HashMap<String, String>>,
    {
        fn get(&self, key: &str) -> Result<String, std::env::VarError> {
            self.borrow()
                .get(key)
                .ok_or(std::env::VarError::NotPresent)
                .cloned()
        }
    }

    impl<T> WriteEnv for T
    where
        T: std::borrow::BorrowMut<HashMap<String, String>>,
    {
        fn set(&mut self, key: String, value: String) {
            self.borrow_mut().insert(key, value);
        }
    }

    pub struct StdEnv;

    pub static ENV: StdEnv = StdEnv {};

    impl ReadEnv for StdEnv {
        fn get(&self, key: &str) -> Result<String, std::env::VarError> {
            std::env::var(key)
        }
    }

    impl WriteEnv for StdEnv {
        fn set(&mut self, key: String, value: String) {
            std::env::set_var(key, value);
        }
    }

    pub trait ParseEnv {
        type Error: std::error::Error;

        fn from_str(config: &str) -> Result<HashMap<String, String>, Self::Error>;
        fn from_reader(reader: impl std::io::Read) -> Result<HashMap<String, String>, Self::Error>;
    }
}

pub use env::{Env, ReadEnv, WriteEnv};

pub mod utils {
    /// toPosixPath converts the given path to the posix form.
    ///
    /// On Windows, \\ will be replaced with /.
    pub fn to_posix_path(path: impl AsRef<str>) -> String {
        path.as_ref().replace("\\", "/")
    }

    /// toWin32Path converts the given path to the win32 form.
    ///
    /// On Linux, / will be replaced with \\.
    pub fn to_win32_path(path: impl AsRef<str>) -> String {
        path.as_ref().replace("/", "\\")
    }

    /// toPlatformPath converts the given path to a platform-specific path.
    ///
    /// It does this by replacing instances of / and \ with
    /// the platform-specific path separator.
    pub fn to_platform_path(path: impl AsRef<str>) -> String {
        path.as_ref()
            .replace("/", std::path::MAIN_SEPARATOR_STR)
            .replace("\\", std::path::MAIN_SEPARATOR_STR)
    }

    pub fn escape_data(data: impl AsRef<str>) -> String {
        data.as_ref()
            .replace('%', "%25")
            .replace('\r', "%0D")
            .replace('\n', "%0A")
    }

    pub fn escape_property(prop: impl AsRef<str>) -> String {
        prop.as_ref()
            .replace('%', "%25")
            .replace('\r', "%0D")
            .replace('\n', "%0A")
            .replace(':', "%3A")
            .replace(',', "%2C")
    }
}

pub mod summary {
    pub const SUMMARY_ENV_VAR: &str = "GITHUB_STEP_SUMMARY";
    pub const SUMMARY_DOCS_URL: &str = "https://docs.github.com/actions/using-workflows/workflow-commands-for-github-actions#adding-a-job-summary";

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    pub struct SummaryTableCell {
        /// Cell content
        pub data: String,
        /// Render cell as header
        pub header: bool,
        /// Number of columns the cell extends
        pub colspan: usize,
        /// Number of rows the cell extends
        pub rowspan: usize,
    }

    impl SummaryTableCell {
        pub fn new(data: String) -> Self {
            Self {
                data,
                ..Self::default()
            }
        }

        pub fn header(data: String) -> Self {
            Self {
                data,
                header: true,
                ..Self::default()
            }
        }
    }

    impl Default for SummaryTableCell {
        fn default() -> Self {
            Self {
                data: "".to_string(),
                header: false,
                colspan: 1,
                rowspan: 1,
            }
        }
    }

    #[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
    pub struct SummaryImageOptions {
        /// The width of the image in pixels.
        width: Option<usize>,

        /// The height of the image in pixels.
        height: Option<usize>,
    }

    // todo: finish porting the summary stuff
    // finish the proc macro, and test it!
    // continue with the cache stuff?
}

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
pub fn export_var(name: impl AsRef<str>, value: impl ToString) -> Result<(), ValueError> {
    std::env::set_var(name.as_ref(), value.to_string());

    if std::env::var("GITHUB_ENV").and_then(not_empty).is_ok() {
        let message = prepare_kv_message(name.as_ref(), &value.to_string())?;
        issue_file_command("ENV", &message).unwrap();
        return Ok(());
    }

    issue(
        CommandBuilder::new("set-env", value)
            .property("name", name.as_ref())
            .build(),
    );
    Ok(())
}

/// Registers a secret which will get masked from logs.
pub fn set_secret(secret: impl ToString) {
    issue(CommandBuilder::new("add-mask", secret.to_string()).build());
}

pub fn append_to_path(path: impl AsRef<Path>) -> Result<(), std::env::JoinPathsError> {
    if let Some(old_path) = std::env::var_os("PATH") {
        let paths = [path.as_ref().to_path_buf()]
            .into_iter()
            .chain(std::env::split_paths(&old_path));
        let new_path = std::env::join_paths(paths)?;
        std::env::set_var("PATH", &new_path);
    }
    Ok(())
}

/// Prepends inputPath to the PATH.
///
/// For this action and future actions.
pub fn add_path(path: impl AsRef<Path>) -> Result<(), std::env::JoinPathsError> {
    let path_string = path.as_ref().to_string_lossy();
    if std::env::var("GITHUB_PATH").and_then(not_empty).is_ok() {
        issue_file_command("PATH", &path_string).unwrap();
    } else {
        issue(CommandBuilder::new("add-path", path_string).build());
    }

    append_to_path(path)
}

pub trait Parse {
    type Input;
    fn parse<E: ReadEnv>(env: &E) -> HashMap<Self::Input, Option<String>>;
}

pub trait ParseInput: Sized {
    type Error: std::error::Error;

    fn parse(value: String) -> Result<Self, Self::Error>;
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Hash, Clone)]
pub enum ParseError {
    #[error("invalid boolean value \"{0}\"")]
    Bool(String),
}

impl ParseInput for String {
    type Error = std::convert::Infallible;
    fn parse(value: String) -> Result<Self, Self::Error> {
        Ok(value)
    }
}

impl ParseInput for bool {
    type Error = ParseError;
    fn parse(value: String) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "yes" => Ok(true),
            "true" => Ok(true),
            "t" => Ok(true),
            "no" => Ok(false),
            "false" => Ok(false),
            "f" => Ok(false),
            _ => Err(ParseError::Bool(value)),
        }
    }
}

/// Gets the value of an input.
///
/// Attempts to parse as T.
pub fn get_input<T>(name: impl AsRef<str>) -> Result<Option<T>, <T as ParseInput>::Error>
where
    T: ParseInput,
{
    match get_raw_input(&env::ENV, name) {
        Ok(input) => Some(T::parse(input)).transpose(),
        Err(_) => Ok(None),
    }
}

pub fn not_empty(value: String) -> Result<String, std::env::VarError> {
    if value.is_empty() {
        Err(std::env::VarError::NotPresent)
    } else {
        Ok(value)
    }
}

/// Gets the raw value of an input.
pub fn get_raw_input(
    env: &impl ReadEnv,
    name: impl AsRef<str>,
) -> Result<String, std::env::VarError> {
    env.get(&input_env_var(name.as_ref())) // .and_then(not_empty)
}

/// Gets the value of an input.
///
/// Attempts to parse as T.
pub fn get_input_from<T>(
    env: &impl ReadEnv,
    name: impl AsRef<str>,
) -> Result<Option<T>, <T as ParseInput>::Error>
where
    T: ParseInput,
{
    match get_raw_input(env, name) {
        Ok(input) => Some(T::parse(input)).transpose(),
        Err(_) => Ok(None),
    }
}

/// Gets the values of an multiline input.
pub fn get_multiline_input<'a>(name: impl AsRef<str>) -> Result<Vec<String>, std::env::VarError> {
    let value = get_raw_input(&env::ENV, name)?;
    Ok(value.lines().map(ToOwned::to_owned).collect())
}

/// Enables or disables the echoing of commands into stdout for the rest of the step.
///
/// Echoing is disabled by default if ACTIONS_STEP_DEBUG is not set.
pub fn set_command_echo(enabled: bool) {
    issue(CommandBuilder::new("echo", if enabled { "on" } else { "off" }).build());
}

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
    pub fn new(command: impl ToString, message: impl ToString) -> Self {
        Self {
            command: command.to_string(),
            message: message.to_string(),
            props: HashMap::new(),
        }
    }

    pub fn property(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.props.insert(key.to_string(), value.to_string());
        self
    }

    pub fn properties(mut self, props: HashMap<String, String>) -> Self {
        self.props.extend(props.into_iter());
        self
    }

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

pub fn issue(cmd: Command) {
    println!("{}", cmd);
}

#[derive(thiserror::Error, Debug)]
pub enum ValueError {
    #[error("should not contain delimiter `{delimiter}`")]
    ContainsDelimiter { delimiter: String },
}

#[derive(thiserror::Error, Debug)]
pub enum FileCommandError {
    #[error("missing environment valirable for file command {cmd}")]
    Missing {
        source: std::env::VarError,
        cmd: String,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Value(#[from] ValueError),
}

pub fn issue_file_command(
    command: impl AsRef<str>,
    message: impl AsRef<str>,
) -> Result<(), FileCommandError> {
    use std::io::Write;
    let file_path = std::env::var(format!("GITHUB_{}", command.as_ref())).map_err(|source| {
        FileCommandError::Missing {
            source,
            cmd: command.as_ref().to_string(),
        }
    })?;
    let file = std::fs::OpenOptions::new()
        .append(true)
        .write(true)
        .open(file_path)?;
    let mut file = std::io::BufWriter::new(file);
    writeln!(file, "{}", message.as_ref())?;
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

impl From<AnnotationProperties> for HashMap<String, String> {
    fn from(props: AnnotationProperties) -> Self {
        let values = [
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
        .filter_map(|(k, v)| match v {
            Some(v) => Some((k, v)),
            None => None,
        });
        Self::from_iter(values)
    }
}

/// Adds an error issue.
pub fn issue_level(level: LogLevel, message: impl ToString, props: Option<AnnotationProperties>) {
    let props = props.unwrap_or_default();
    issue(
        CommandBuilder::new(level, message)
            .properties(props.into())
            .build(),
    );
}

// /// Writes debug message to user log.
// pub fn debug(message: impl std::fmt::Display) {
//     issue_command("debug", message, HashMap::new())
// }

/// Adds an error issue.
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
/// Output until the next group_end will be foldable in this group.
pub fn start_group(name: impl std::fmt::Display) {
    issue(CommandBuilder::new("group", name).build())
}

/// End an output group.
pub fn end_group() {
    issue(CommandBuilder::new("endgroup", "").build())
}

/// Saves state for current action, the state can only be retrieved by this action's post job execution.
pub fn save_state(name: String, value: impl std::fmt::Display) {
    if std::env::var("GITHUB_STATE").and_then(not_empty).is_ok() {
        let message = prepare_kv_message(&name, &value.to_string()).unwrap();
        issue_file_command("STATE", &message).unwrap();
        return;
    }

    issue(
        CommandBuilder::new("save-state", value)
            .property("name", name)
            .build(),
    );
}

/// Gets the value of an state set by this action's main execution.
pub fn get_state(name: String) -> Option<String> {
    std::env::var(format!("STATE_{}", name)).ok()
}

/// Wrap an asynchronous function call in a group.
///
/// Returns the same type as the function itself.
pub async fn group<T>(
    name: impl std::fmt::Display,
    fut: impl std::future::Future<Output = T>,
) -> T {
    start_group(name);
    let res: T = fut.await;

    end_group();
    res
}

#[cfg(test)]
mod tests {
    use super::Env;

    #[test]
    fn test_env() {
        let input_name = "SOME_NAME";
        let env = Env::from_iter([(input_name.to_string(), "SET".to_string())]);
        dbg!(&env);
        assert_eq!(env.get("INPUT_SOME_NAME"), Some(&"SET".to_string()));
    }

    #[test]
    fn test_get_non_empty_input() {
        let input_name = "SOME_NAME";
        let env = Env::from_iter([(input_name.to_string(), "SET".to_string())]);
        dbg!(&env);
        assert_eq!(
            super::get_input_from::<String>(&env, input_name),
            Ok(Some("SET".to_string()))
        );
    }

    #[test]
    fn test_get_empty_input() {
        let input_name = "SOME_NAME";
        let mut env = Env::from_iter([]);
        assert_eq!(
            super::get_input_from::<String>(&env, input_name),
            Ok(None),
        );

        env.insert(input_name.to_string(), "".to_string());
        assert_eq!(
            super::get_input_from::<String>(&env, input_name),
            Ok(None),
        );
    }
}
