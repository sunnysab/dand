use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const SITE_CONFIG_FILE: &str = "dand.toml";

#[derive(Clone, Serialize, Deserialize)]
/// Site basic information and description.
pub struct SiteConf {
    /// Blog name
    pub title: String,
    /// Complete home url to the site, e.g. https://example.com/
    pub home: String,
    /// Site description in one sentence
    pub description: String,
}

impl SiteConf {
    pub fn load(conf_path: impl AsRef<Path>) -> Result<SiteConf> {
        let conf_path: &Path = conf_path.as_ref();
        let toml_content = std::fs::read_to_string(conf_path)
            .with_context(|| format!("failed to load site config: {}", conf_path.display()))?;

        toml::from_str(&toml_content)
            .with_context(|| format!("failed to parse site config: {}", conf_path.display()))
    }
}
