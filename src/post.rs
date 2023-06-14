use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use flag::PostFlag;
pub use parser::PostParser;

use crate::post::tag::SharedTag;

use super::datetime::datetime_serde;
use super::discover::PostReference;

mod flag;
mod parser;
pub mod tag;

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
    tags: Vec<String>,
}

pub struct PostMetadata {
    pub title: String,
    pub publish_date: DateTime<Local>,
    pub modify_date: DateTime<Local>,
    pub flag: PostFlag,
    pub tags: Vec<SharedTag>,
}

pub struct Post {
    pub reference: PostReference,
    raw_content: Rc<Box<str>>,

    pub metadata: PostMetadata,
    body_start_pos: usize,
}

impl Post {
    pub fn body(&self) -> &str {
        &self.raw_content[self.body_start_pos..]
    }

    pub fn expected_url(&self) -> String {
        let site = &super::site().home;
        let relative_name = &self.reference.relative_name;

        format!("{site}{relative_name}")
    }
}

#[derive(Clone)]
pub struct SharedPost(pub Rc<RefCell<Post>>);

impl SharedPost {
    pub fn from(post: Post) -> Self {
        SharedPost(Rc::new(RefCell::new(post)))
    }

    pub fn add_tag(&self, tag: &SharedTag) {
        self.0.borrow_mut().metadata.tags.push(tag.clone());
    }
}
