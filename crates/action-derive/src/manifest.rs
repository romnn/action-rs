use std::collections::HashMap;
use std::path::Path;

#[derive(PartialEq, Eq, Hash, Debug, serde::Deserialize)]
pub struct Input {
    pub description: Option<String>,
    #[serde(rename(deserialize = "deprecationMessage"))]
    pub deprecation_message: Option<String>,
    pub default: Option<String>,
    pub required: Option<bool>,
}

#[derive(PartialEq, Eq, Hash, Debug, serde::Deserialize)]
pub struct Output {
    pub description: Option<String>,
}

#[derive(PartialEq, Eq, Hash, Debug, serde::Deserialize)]
pub struct Branding {
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(PartialEq, Eq, Debug, serde::Deserialize)]
pub struct Manifest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub branding: Option<Branding>,

    #[serde(default)]
    pub inputs: HashMap<String, Input>,
    #[serde(default)]
    pub outputs: HashMap<String, Output>,
}

impl Manifest {
    pub fn from_action_yml(path: impl AsRef<Path>) -> Self {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(path.as_ref())
            .unwrap();
        let reader = std::io::BufReader::new(file);
        serde_yaml::from_reader(reader).unwrap()
    }
}
