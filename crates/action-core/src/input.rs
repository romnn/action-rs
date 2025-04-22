use crate::{env, utils::not_empty};
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Input<'a> {
    pub description: Option<&'a str>,
    pub deprecation_message: Option<&'a str>,
    pub default: Option<&'a str>,
    pub required: Option<bool>,
}

pub fn env_var_name(name: impl AsRef<OsStr>) -> OsString {
    // const PREFIX: &[u8; 6] = b"INPUT_";
    // const PREFIX: &OsStr = &OsStr::new("INPUT_");
    let name = name.as_ref();
    let prefix: &OsStr = &OsStr::new("INPUT_");
    let mut out = OsString::from(prefix);
    if name.as_bytes().starts_with(prefix.as_bytes()) {
        // out.push(name[..prefix.len()].as_ref());
        // out.push(&name.as_bytes()[..prefix.len()]);
        // out.push(&name[..prefix.len()]);
        out.push(OsStr::from_bytes(&name.as_bytes()[..prefix.len()]));
    } else {
        out.push(name);
    }
    out
    // let mut var = name.as_ref().to_string_lossy().to_string();
    // if !var.starts_with("INPUT_") {
    //     var = format!("INPUT_{var}");
    // }
    // var = var.replace(' ', "_").to_uppercase();
    // var.try_into()
}

pub trait Parse: Sized {
    type Error: std::error::Error;

    /// Parse input string to type T.
    ///
    /// # Errors
    /// When the string value cannot be parsed as `Self`.
    fn parse(value: OsString) -> Result<Self, Self::Error>;
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Hash, Clone)]
pub enum ParseError {
    #[error("invalid boolean value \"{0:?}\"")]
    Bool(OsString),
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
        match value.to_ascii_lowercase().as_os_str().as_bytes() {
            b"yes" | b"true" | b"t" => Ok(true),
            b"no" | b"false" | b"f" => Ok(false),
            _ => Err(ParseError::Bool(value)),
        }
    }
}

// /// Gets the value of an input.
// ///
// /// Attempts to parse as T if a value is present, other returns `Ok(None)`.
// ///
// /// # Errors
// /// If the variable cannot be parsed.
// pub fn get<T>(
//     env: impl env::Read,
//     name: impl AsRef<OsStr>,
// ) -> Result<Option<T>, <T as Parse>::Error>
// where
//     T: Parse,
// {
//     match get_raw(env, name) {
//         Some(input) => Some(T::parse(input)).transpose(),
//         None => Ok(None),
//     }
// }

pub trait SetInput {
    /// Sets an input.
    fn set_input(&self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>);
}

impl<E> SetInput for E
where
    E: env::Write,
{
    fn set_input(&self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        self.set(env_var_name(name.as_ref()), value)
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

// /// Gets the value of an input from an environment.
// ///
// /// Attempts to parse as T if a value is present, other returns `Ok(None)`.
// ///
// /// # Errors
// /// If the variable cannot be parsed.
// pub fn get_from<T>(
//     env: &impl env::Read,
//     name: impl AsRef<OsStr>,
// ) -> Result<Option<T>, <T as Parse>::Error>
// where
//     T: Parse,
// {
//     match get_raw(env, name) {
//         Some(input) => Some(T::parse(input)).transpose(),
//         None => Ok(None),
//     }
// }

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
    use crate::env::EnvMap;
    use similar_asserts::assert_eq as sim_assert_eq;

    #[test]
    fn test_get_non_empty_input() {
        let env = EnvMap::default();
        let input_name = "some-input";
        env.set_input(input_name, "SET");
        sim_assert_eq!(env.get_input(input_name), Some("SET".into()));
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
