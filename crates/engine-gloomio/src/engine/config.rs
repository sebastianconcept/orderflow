//! Name and version of a Gloomio matching engine.
//!
//! When Engine receives a parameter set, it uses this module so name and
//! version are complete.

/// EngineConfig is the name and version of a Gloomio matching engine.
/// When Engine is constructed, it uses this type so name and version are a
/// complete set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineConfig {
    pub name: String,
    pub version: String,
}

impl EngineConfig {
    /// Answers an EngineConfig from name and version.
    pub fn new(name: &str, version: &str) -> Self {
        EngineConfig {
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    /// Answers an EngineConfig with test name and version.
    #[cfg(test)]
    pub fn for_test() -> Self {
        EngineConfig {
            name: "Test gloomio Engine".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    /// Answers the default name and version for this engine.
    pub fn default_config() -> Self {
        EngineConfig {
            name: "Default gloomio Matching Engine".to_string(),
            version: "0.1.0".to_string(),
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self::default_config()
    }
}
