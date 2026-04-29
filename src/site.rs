use std::path::Path;

use anyhow::{Context, Result};
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
/// Site basic information and description.
pub struct SiteConf {
    /// Blog name
    pub title: String,
    /// Complete home url to the site, e.g. https://example.com/
    pub home: String,
    /// Site description in one sentence
    pub description: String,
    #[serde(default)]
    pub footer: FooterConf,
    #[serde(default)]
    pub post: PostConf,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FooterConf {
    #[serde(default = "default_footer_start_year")]
    pub start_year: u16,
    #[serde(default = "default_footer_beian")]
    pub beian: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PostConf {
    #[serde(default)]
    pub normalize_slug: bool,
}

#[derive(Clone, Serialize)]
pub struct FooterView {
    pub year_label: String,
    pub beian: String,
}

impl Default for FooterConf {
    fn default() -> Self {
        Self {
            start_year: default_footer_start_year(),
            beian: default_footer_beian(),
        }
    }
}

impl SiteConf {
    pub fn load(conf_path: impl AsRef<Path>) -> Result<SiteConf> {
        let conf_path: &Path = conf_path.as_ref();
        let toml_content = std::fs::read_to_string(conf_path)
            .with_context(|| format!("failed to load site config: {}", conf_path.display()))?;

        toml::from_str(&toml_content)
            .with_context(|| format!("failed to parse site config: {}", conf_path.display()))
    }

    pub fn footer_view(&self) -> FooterView {
        let current_year = Local::now().year() as u16;
        let year_label = if self.footer.start_year < current_year {
            format!("{} - {}", self.footer.start_year, current_year)
        } else {
            self.footer.start_year.to_string()
        };

        FooterView {
            year_label,
            beian: self.footer.beian.clone(),
        }
    }
}

fn default_footer_start_year() -> u16 {
    2020
}

fn default_footer_beian() -> String {
    "沪ICP备20002980号".to_string()
}
