use crate::{env, utils::not_empty};
use std::ffi::{OsStr, OsString};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Input<'a> {
    pub description: Option<&'a str>,
    pub deprecation_message: Option<&'a str>,
    pub default: Option<&'a str>,
    pub required: Option<bool>,
}

#[cfg(not(target_family = "unix"))]
pub fn env_var_name(name: impl AsRef<OsStr>) -> OsString {
    let name = name.as_ref().to_string_lossy();
    let out: OsString = if name.starts_with("INPUT_") {
        name.to_string().into()
    } else {
        format!("INPUT_{name}").into()
    };
    out.to_ascii_uppercase()
}

#[cfg(target_family = "unix")]
pub fn env_var_name(name: impl AsRef<OsStr>) -> OsString {
    use std::os::unix::ffi::OsStrExt;
    let name = name.as_ref();
    let prefix: &OsStr = OsStr::new("INPUT_");
    let mut out = OsString::from(prefix);
    if name
        .as_encoded_bytes()
        .starts_with(prefix.as_encoded_bytes())
    {
        out.push(OsStr::from_bytes(&name.as_encoded_bytes()[prefix.len()..]));
    } else {
        out.push(name);
    }
    out.to_ascii_uppercase()
}

pub trait Parse: Sized {
    type Error: std::error::Error;

    /// Parse input string to type T.
    ///
    /// # Errors
    /// When the string value cannot be parsed as `Self`.
    fn parse(value: OsString) -> Result<Self, Self::Error>;
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum ParseError {
    #[error("invalid boolean value \"{0:?}\"")]
    Bool(OsString),
    #[error("invalid integer value \"{value:?}\"")]
    Int {
        value: OsString,
        #[source]
        source: std::num::ParseIntError,
    },
}

impl Parse for String {
    type Error = std::convert::Infallible;

    fn parse(value: OsString) -> Result<Self, Self::Error> {
        Ok(value.to_string_lossy().to_string())
    }
}

impl Parse for OsString {
    type Error = std::convert::Infallible;

    fn parse(value: OsString) -> Result<Self, Self::Error> {
        Ok(value)
    }
}

impl Parse for bool {
    type Error = ParseError;
    fn parse(value: OsString) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_os_str().as_encoded_bytes() {
            b"yes" | b"true" | b"t" => Ok(true),
            b"no" | b"false" | b"f" => Ok(false),
            _ => Err(ParseError::Bool(value)),
        }
    }
}

impl Parse for usize {
    type Error = ParseError;
    fn parse(value: OsString) -> Result<Self, Self::Error> {
        value
            .to_string_lossy()
            .to_string()
            .parse()
            .map_err(|source| ParseError::Int { value, source })
    }
}

pub trait SetInput {
    /// Sets an input.
    fn set_input(&self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>);
}

impl<E> SetInput for E
where
    E: env::Write,
{
    fn set_input(&self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        self.set(env_var_name(name.as_ref()), value);
    }
}

pub trait GetInput {
    /// Gets the raw value of an input.
    fn get_input(&self, name: impl AsRef<OsStr>) -> Option<OsString>;
}

impl<E> GetInput for E
where
    E: env::Read,
{
    fn get_input(&self, name: impl AsRef<OsStr>) -> Option<OsString> {
        self.get(env_var_name(name.as_ref())).and_then(not_empty)
    }
}

pub trait ParseInput {
    /// Parse the value of an input.
    ///
    /// Attempts to parse as T if a value is present, other returns `Ok(None)`.
    ///
    /// # Errors
    /// If the variable cannot be parsed.
    fn parse_input<T>(&self, name: impl AsRef<OsStr>) -> Result<Option<T>, <T as Parse>::Error>
    where
        T: Parse;
}

impl<E> ParseInput for E
where
    E: env::Read,
{
    fn parse_input<T>(&self, name: impl AsRef<OsStr>) -> Result<Option<T>, <T as Parse>::Error>
    where
        T: Parse,
    {
        match self.get_input(name) {
            Some(input) => Some(T::parse(input)).transpose(),
            None => Ok(None),
        }
    }
}

/// Gets the values of an multiline input.
///
/// # Errors
/// If the environment variable is not present.
pub fn get_multiline(env: &impl env::Read, name: impl AsRef<OsStr>) -> Option<Vec<String>> {
    let value = env.get_input(name)?;
    let lines = value
        .to_string_lossy()
        .lines()
        .map(ToOwned::to_owned)
        .collect();
    Some(lines)
}

#[cfg(test)]
mod tests {
    use super::{GetInput, ParseInput, SetInput};
    use crate::env::{EnvMap, Read};
    use similar_asserts::assert_eq as sim_assert_eq;

    #[test]
    fn test_env_name() {
        sim_assert_eq!(super::env_var_name("some-input"), "INPUT_SOME-INPUT");
        sim_assert_eq!(super::env_var_name("INPUT_some-input"), "INPUT_SOME-INPUT");
        sim_assert_eq!(super::env_var_name("INPUT_SOME-INPUT"), "INPUT_SOME-INPUT");
        sim_assert_eq!(
            super::env_var_name("test-INPUT_SOME-INPUT"),
            "INPUT_TEST-INPUT_SOME-INPUT"
        );
    }

    #[test]
    fn test_get_non_empty_input() {
        let env = EnvMap::default();
        env.set_input("some-input", "SET");
        sim_assert_eq!(env.get("INPUT_SOME-INPUT"), Some("SET".into()));
        sim_assert_eq!(env.get_input("some-input"), Some("SET".into()));
    }

    #[test]
    fn test_get_empty_input() {
        let env = EnvMap::default();
        let input_name = "some-input";
        sim_assert_eq!(env.parse_input::<String>(input_name), Ok(None));
        env.set_input(input_name, "");
        sim_assert_eq!(env.parse_input::<String>(input_name), Ok(None));
        env.set_input(input_name, " ");
        sim_assert_eq!(env.parse_input::<String>(input_name), Ok(Some(" ".into())));
    }
}
