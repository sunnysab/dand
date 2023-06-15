use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use flag::PostFlag;

use crate::env::Environment;
use crate::post::tag::SharedTag;

use super::datetime::datetime_serde;
use super::discover::PostReference;

mod flag;
mod parser;
pub mod tag;

#[derive(Debug, Deserialize)]
pub struct RawFrontMatter {
    pub title: String,
    #[serde(default)]
    #[serde(rename = "date", with = "datetime_serde")]
    pub publish_date: Option<DateTime<Local>>,
    #[serde(default)]
    #[serde(rename = "updated", with = "datetime_serde")]
    pub modify_date: Option<DateTime<Local>>,
    pub is_hidden: Option<bool>,
    pub is_draft: Option<bool>,
    pub use_latex: Option<bool>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub publish_date: DateTime<Local>,
    pub modify_date: DateTime<Local>,
    pub flag: PostFlag,
    pub tags: Vec<SharedTag>,
}

#[derive(Debug)]
pub struct Post {
    pub reference: PostReference,
    pub raw_content: Rc<Box<str>>,

    pub metadata: Metadata,
    pub body: &'static str,
}

impl Post {
    pub fn expected_url(&self) -> String {
        let site = &super::site().home;
        let relative_name = &self.reference.relative_name;

        format!("{site}{relative_name}")
    }
}

#[derive(Debug, Clone)]
pub struct SharedPost(pub Rc<RefCell<Post>>);

impl SharedPost {
    pub fn from(post: Post) -> Self {
        SharedPost(Rc::new(RefCell::new(post)))
    }

    pub fn add_tag(&self, tag: &SharedTag) {
        self.0.borrow_mut().metadata.tags.push(tag.clone());
    }
}

pub struct LoadedPost {
    pub reference: PostReference,
    pub full_content: Box<str>,

    pub front_matter: RawFrontMatter,
    pub body: &'static str,
}

pub fn load(reference: PostReference) -> Result<LoadedPost> {
    let path = &reference.full_path;

    let full_content = std::fs::read_to_string(path)?.into_boxed_str();
    let (front_matter, body) = parser::split(&full_content)?;

    let body: &'static str = unsafe { std::mem::transmute(body) };
    Ok(LoadedPost {
        reference,
        full_content,
        front_matter,
        body,
    })
}

impl LoadedPost {
    pub fn build(self, env: &mut Environment) -> Result<SharedPost> {
        let LoadedPost {
            reference,
            full_content,
            front_matter,
            body,
        } = self;

        let flag = PostFlag::new()
            .with_draft_flag(front_matter.is_draft.unwrap_or(false))
            .with_hidden_flag(front_matter.is_hidden.unwrap_or(true))
            .with_latex_enabled_flag(front_matter.use_latex.unwrap_or(false));
        let metadata = Metadata {
            title: front_matter.title,
            publish_date: front_matter.publish_date.unwrap_or(reference.created),
            modify_date: front_matter.modify_date.unwrap_or(reference.modified),
            flag,
            tags: vec![],
        };
        let post = Post {
            reference,
            raw_content: Rc::new(full_content),
            metadata,
            body,
        };

        let shared_post = SharedPost::from(post);
        let tags = env.get_tags_mut();
        for tag in &front_matter.tags {
            tags.add(tag, &shared_post);
        }
        env.add_post(shared_post.clone());
        Ok(shared_post)
    }
}
