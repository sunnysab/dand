use std::fmt::{Debug, Display, Formatter};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use super::datetime::datetime_serde;
use super::discover::PostReference;

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

#[derive(Clone, Copy, strum::Display, strum::EnumIter)]
enum PostFlagEnum {
    Hidden = 1,            // 0x01
    Draft = 1 << 1,        // 0x02
    LatexEnabled = 1 << 2, // 0x04
}

// Maybe smart pointer is needed?
#[derive(Debug, Default, Serialize, Deserialize)]
struct Tag {}

pub struct PostFlag(u32);

impl PostFlag {
    pub fn new() -> Self {
        PostFlag(0)
    }

    pub fn is_hidden(&self) -> bool {
        self.0 & (PostFlagEnum::Hidden as u32) != 0
    }

    pub fn is_draft(&self) -> bool {
        self.0 & (PostFlagEnum::Draft as u32) != 0
    }

    pub fn is_latex_enabled(&self) -> bool {
        self.0 & (PostFlagEnum::LatexEnabled as u32) != 0
    }

    pub fn set_hidden(&mut self) {
        self.0 |= PostFlagEnum::Hidden as u32;
    }

    pub fn set_draft(&mut self) {
        self.0 |= PostFlagEnum::Draft as u32;
    }

    pub fn set_latex_enabled(&mut self) {
        self.0 |= PostFlagEnum::LatexEnabled as u32;
    }
}

impl Display for PostFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use strum::IntoEnumIterator;

        let mut v = vec![];
        for flag in PostFlagEnum::iter() {
            if self.0 & (flag as u32) != 0 {
                v.push(flag.to_string());
            }
        }
        let result = format!("[{}]", v.join(", "));
        std::fmt::Display::fmt(&result, f)
    }
}

impl Debug for PostFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

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

pub struct PostParser {
    reference: PostReference,
    raw_content: Box<str>,
}

impl PostParser {
    pub fn open(reference: PostReference) -> Result<Self> {
        let path = &reference.full_path;
        let raw_content = std::fs::read_to_string(path)?.into_boxed_str();

        let result = Self {
            reference,
            raw_content,
        };
        Ok(result)
    }

    fn parse_header(&self, yaml_header: &str) -> Result<PostMetadata> {
        let raw_metadata: PostMetadataRaw = serde_yaml::from_str(yaml_header)?;

        let mut flag = PostFlag::new();
        if raw_metadata.is_draft.unwrap_or(false) {
            flag.set_draft();
        }
        if raw_metadata.is_hidden.unwrap_or(false) {
            flag.set_hidden();
        }
        if raw_metadata.use_latex.unwrap_or(false) {
            flag.set_latex_enabled();
        }

        let PostMetadataRaw {
            title,
            publish_date,
            modify_date,
            ..
        } = raw_metadata;
        let publish_date = publish_date.unwrap_or(self.reference.created);
        let modify_date = modify_date.unwrap_or(self.reference.modified);
        let result = PostMetadata {
            title,
            publish_date,
            modify_date,
            flag,
            tag: vec![],
        };
        Ok(result)
    }

    fn split(&self) -> Result<(PostMetadata, usize)> {
        let raw_content = self.raw_content.trim_start();

        if !raw_content.starts_with("---\n") {
            return Err(anyhow!("metadata undetected."));
        }
        // 4 is the length of "---\n"
        // raw_content[4..].find() skipped the first four chars, so the index that find() returned should be added 4, makes
        // the index is the offset relative to raw_content[0].
        let end = 4 + raw_content[4..]
            .find("\n---\n")
            .ok_or_else(|| anyhow!("metadata end is not found."))?;
        let header = &raw_content[4..end];

        let metadata = self
            .parse_header(header)
            .with_context(|| format!("failed to parse header: \n{}", header))?;
        Ok((metadata, end + 9))
    }

    pub fn parse(self) -> Result<Post> {
        let (metadata, body_index) = self.split()?;
        let PostParser {
            reference,
            raw_content,
            ..
        } = self;

        Ok(Post {
            reference,
            raw_content,
            metadata,
            body_start_pos: body_index,
        })
    }
}
