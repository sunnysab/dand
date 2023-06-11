use std::fmt::Debug;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use flag::PostFlag;
pub use parser::PostParser;

use super::datetime::datetime_serde;
use super::discover::PostReference;

mod flag;
mod parser;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMetadataRaw {
    title: String,
    #[serde(rename = "date", with = "datetime_serde")]
    publish_date: Option<DateTime<Local>>,
    #[serde(rename = "updated", with = "datetime_serde")]
    modify_date: Option<DateTime<Local>>,
    is_hidden: Option<bool>,
    is_draft: Option<bool>,
    use_latex: Option<bool>,
    #[serde(default)]
    tag: Vec<String>,
}

// Maybe smart pointer is needed?
#[derive(Debug, Default, Serialize, Deserialize)]
struct Tag {}

#[derive(Debug)]
pub struct PostMetadata {
    title: String,
    publish_date: DateTime<Local>,
    modify_date: DateTime<Local>,
    flag: PostFlag,

    tag: Vec<Tag>,
}

#[derive(Debug)]
pub struct Post {
    reference: PostReference,
    raw_content: Box<str>,

    metadata: PostMetadata,
    body_start_pos: usize,
}

impl Post {
    pub fn body(&self) -> &str {
        &self.raw_content[self.body_start_pos..]
    }
}
