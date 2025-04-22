use parking_lot::Mutex;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct EnvMap {
    inner: Arc<Mutex<HashMap<OsString, OsString>>>,
}

impl<K, V> FromIterator<(K, V)> for EnvMap
where
    K: Into<OsString>,
    V: Into<OsString>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self::new(HashMap::from_iter(
            iter.into_iter().map(|(k, v)| (k.into(), v.into())),
        ))
    }
}

impl EnvMap {
    #[must_use]
    pub fn new(inner: HashMap<OsString, OsString>) -> Self {
        let inner = Arc::new(Mutex::new(inner));
        Self { inner }
    }
}

pub trait Read {
    /// Get value from environment.
    ///
    /// # Errors
    /// When the environment variable is not present.
    fn get<K>(&self, key: K) -> Option<OsString>
    where
        K: AsRef<OsStr>;
}

pub trait Write {
    /// Set value for environment.
    fn set<K, V>(&self, key: K, value: V)
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>;
}

impl Read for EnvMap {
    fn get<K>(&self, key: K) -> Option<OsString>
    where
        K: AsRef<OsStr>,
    {
        self.inner.lock().get(key.as_ref()).cloned()
    }
}

impl Write for EnvMap {
    fn set<K, V>(&self, key: K, value: V)
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.inner
            .lock()
            .insert(key.as_ref().to_os_string(), value.as_ref().to_os_string());
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OsEnv;

impl Read for OsEnv {
    fn get<K>(&self, key: K) -> Option<OsString>
    where
        K: AsRef<OsStr>,
    {
        std::env::var_os(key)
    }
}

impl Write for OsEnv {
    fn set<K, V>(&self, key: K, value: V)
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        unsafe {
            std::env::set_var(key, value);
        }
    }
}

pub trait Parse {
    type Error: std::error::Error;

    /// Parses environment from a string.
    ///
    /// # Errors
    /// If the input cannot be parsed as a `HashMap<String, String>`.
    fn from_str(config: &str) -> Result<HashMap<String, String>, Self::Error>;

    /// Parses environment from a reader.
    ///
    /// # Errors
    /// If the input cannot be parsed as a `HashMap<String, String>`.
    fn from_reader(reader: impl std::io::Read) -> Result<HashMap<String, String>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::{EnvMap, Read, Write};
    use similar_asserts::assert_eq as sim_assert_eq;

    #[test]
    fn get_env_map() {
        let input_name = "SOME_NAME";
        let env = EnvMap::from_iter([(input_name, "SET")]);
        sim_assert_eq!(env.get(input_name), Some("SET".into()));
    }

    #[test]
    fn set_env_map() {
        let input_name = "SOME_NAME";
        let env = EnvMap::default();
        env.set(input_name, "SET");
        sim_assert_eq!(env.get(input_name), Some("SET".into()));
    }
}
