use std::sync::OnceLock;

use serde::Deserialize;

const SITE_CONFIG_FILE: &str = "site.toml";
static PRIVATE_SITE_CONFIG: OnceLock<Site> = OnceLock::new();

#[derive(Deserialize)]
/// Site basic information and description.
pub struct Site {
    /// Blog name
    pub title: String,
    /// Complete home url to the site, e.g. https://example.com/
    pub home: String,
    /// Site description in one sentence
    pub description: String,
}

fn load_site_config() -> Site {
    let toml_content =
        std::fs::read_to_string(SITE_CONFIG_FILE).expect("Unable to load site config.");

    toml::from_str(&toml_content).expect("Failed to parse site config.")
}

pub fn get() -> &'static Site {
    PRIVATE_SITE_CONFIG.get_or_init(load_site_config)
}
