#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub name: String,
    pub version: String,
}

impl Config {
    pub fn new(name: &str, version: &str) -> Self {
        Config {
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    #[cfg(test)]
    pub fn for_test() -> Self {
        Config {
            name: "Test tokio Engine".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    pub fn default_config() -> Self {
        Config {
            name: "Default tokio Matching Engine".to_string(),
            version: "0.1.0".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_config()
    }
}
