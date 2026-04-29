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

#[derive(Clone, Serialize, Deserialize)]
pub struct PostConf {
    #[serde(default)]
    pub normalize_slug: bool,
    #[serde(default = "default_publish_dirs")]
    pub publish_dirs: Vec<String>,
    #[serde(default = "default_draft_dirs")]
    pub draft_dirs: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
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

impl Default for PostConf {
    fn default() -> Self {
        Self {
            normalize_slug: false,
            publish_dirs: default_publish_dirs(),
            draft_dirs: default_draft_dirs(),
            ignore: Vec::new(),
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

impl PostConf {
    pub fn is_publish_path(&self, relative_path: &Path) -> bool {
        path_matches_prefixes(relative_path, &self.publish_dirs)
    }

    pub fn is_draft_path(&self, relative_path: &Path) -> bool {
        path_matches_prefixes(relative_path, &self.draft_dirs)
    }

    pub fn should_hide_path(&self, relative_path: &Path) -> bool {
        !self.is_publish_path(relative_path) && !self.is_draft_path(relative_path)
    }

    pub fn is_ignored_path(&self, relative_path: &Path) -> bool {
        if self.ignore.is_empty() {
            return false;
        }

        let normalized = normalize_relative_path(relative_path);
        if normalized.is_empty() {
            return false;
        }

        self.ignore
            .iter()
            .any(|pattern| glob_match_path(pattern, &normalized))
    }
}

fn default_footer_start_year() -> u16 {
    2020
}

fn default_publish_dirs() -> Vec<String> {
    vec!["_posts".to_string()]
}

fn default_draft_dirs() -> Vec<String> {
    vec!["_drafts".to_string()]
}

fn default_footer_beian() -> String {
    "沪ICP备20002980号".to_string()
}

fn normalize_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn path_matches_prefixes(path: &Path, prefixes: &[String]) -> bool {
    if prefixes.is_empty() {
        return false;
    }

    let segments: Vec<String> = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();

    prefixes
        .iter()
        .filter(|prefix| !prefix.is_empty())
        .map(|prefix| prefix.split('/').collect::<Vec<_>>())
        .any(|prefix_segments| {
            prefix_segments.len() <= segments.len()
                && prefix_segments
                    .iter()
                    .zip(segments.iter())
                    .all(|(expected, actual)| expected == actual)
        })
}

fn glob_match_path(pattern: &str, path: &str) -> bool {
    let pattern_segments: Vec<&str> = if pattern.is_empty() {
        Vec::new()
    } else {
        pattern.split('/').collect()
    };
    let path_segments: Vec<&str> = if path.is_empty() {
        Vec::new()
    } else {
        path.split('/').collect()
    };
    glob_match_segments(&pattern_segments, &path_segments)
}

fn glob_match_segments(patterns: &[&str], segments: &[&str]) -> bool {
    match patterns.split_first() {
        None => segments.is_empty(),
        Some((&"**", rest)) => {
            glob_match_segments(rest, segments)
                || (!segments.is_empty() && glob_match_segments(patterns, &segments[1..]))
        }
        Some((pattern, rest)) => {
            !segments.is_empty()
                && glob_match_component(pattern, segments[0])
                && glob_match_segments(rest, &segments[1..])
        }
    }
}

fn glob_match_component(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    let mut dp = vec![vec![false; text_chars.len() + 1]; pattern_chars.len() + 1];
    dp[0][0] = true;

    for i in 0..pattern_chars.len() {
        match pattern_chars[i] {
            '*' => {
                for j in 0..=text_chars.len() {
                    dp[i + 1][j] |= dp[i][j];
                    if j < text_chars.len() {
                        dp[i + 1][j + 1] |= dp[i + 1][j];
                    }
                }
            }
            '?' => {
                for j in 0..text_chars.len() {
                    if dp[i][j] {
                        dp[i + 1][j + 1] = true;
                    }
                }
            }
            ch => {
                for j in 0..text_chars.len() {
                    if dp[i][j] && text_chars[j] == ch {
                        dp[i + 1][j + 1] = true;
                    }
                }
            }
        }
    }

    dp[pattern_chars.len()][text_chars.len()]
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{PostConf, default_draft_dirs, default_publish_dirs};

    #[test]
    fn post_ignore_supports_glob_patterns() {
        let post = PostConf {
            normalize_slug: false,
            publish_dirs: default_publish_dirs(),
            draft_dirs: default_draft_dirs(),
            ignore: vec![
                "_drafts/**".to_string(),
                "_posts/tmp-*".to_string(),
                "notes/private.md".to_string(),
            ],
        };

        assert!(post.is_ignored_path(Path::new("_drafts")));
        assert!(post.is_ignored_path(Path::new("_drafts/draft.md")));
        assert!(post.is_ignored_path(Path::new("_posts/tmp-secret.md")));
        assert!(post.is_ignored_path(Path::new("notes/private.md")));
        assert!(!post.is_ignored_path(Path::new("_posts/public.md")));
    }

    #[test]
    fn post_dir_semantics_are_configurable() {
        let post = PostConf {
            normalize_slug: false,
            publish_dirs: vec!["posts".to_string(), "content/public".to_string()],
            draft_dirs: vec!["drafts".to_string()],
            ignore: Vec::new(),
        };

        assert!(post.is_publish_path(Path::new("posts/hello.md")));
        assert!(post.is_publish_path(Path::new("content/public/hello.md")));
        assert!(post.is_draft_path(Path::new("drafts/idea.md")));
        assert!(post.should_hide_path(Path::new("404/index.md")));
        assert!(!post.should_hide_path(Path::new("posts/hello.md")));
        assert!(!post.should_hide_path(Path::new("drafts/idea.md")));
    }
}
