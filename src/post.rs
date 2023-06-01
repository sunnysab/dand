use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use super::discover::PostReference;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMetadataRaw {
    title: String,
    publish_date: Option<DateTime<Local>>,
    modify_date: Option<DateTime<Local>>,
    is_hidden: Option<bool>,
    is_draft: Option<bool>,
    use_latex: Option<bool>,
    #[serde(default)]
    tag: Vec<String>,
}

#[derive(Debug)]
enum PostFlag {
    Hidden = 1 ,           // 0x01
    Draft = 1 << 1,        // 0x02
    LatexEnabled = 1 << 2, // 0x04
}

// Maybe smart pointer is needed?
#[derive(Debug, Default, Serialize, Deserialize)]
struct Tag {

}

#[derive(Debug)]
pub struct PostMetadata {
    title: String,
    publish_date: DateTime<Local>,
    modify_date: DateTime<Local>,
    flag: u32,

    tag: Vec<Tag>,
}

#[derive(Debug)]
pub struct Post {
    reference: PostReference,

    metadata: PostMetadata,
    content: String,
}

pub struct PostBuilder {
    reference: PostReference,

    metadata: Option<PostMetadata>,
    content: Option<String>,
}

impl PostBuilder {

    pub fn new(reference: PostReference) -> Self {
        Self { reference, metadata: None, content: None }
    }

    pub fn read(mut self) -> Result<Self> {
        let path = &self.reference.full_path;
        // Read file content
        self.content = Some(std::fs::read_to_string(path)?);
        Ok(self)
    }

    fn parse_header(&self, yaml_header: &str) -> Result<PostMetadata> {
        let raw_metadata: PostMetadataRaw = serde_yaml::from_str(&yaml_header)?;

        let mut flag = 0;
        if raw_metadata.is_draft.unwrap_or(false) {
            flag |= PostFlag::Draft as u32;
        }
        if raw_metadata.is_hidden.unwrap_or(false) {
            flag |= PostFlag::Hidden as u32;
        }
        if raw_metadata.use_latex.unwrap_or(false) {
            flag |= PostFlag::LatexEnabled as u32;
        }

        let PostMetadataRaw {title, publish_date, modify_date, .. } = raw_metadata;
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

    fn split_markdown(&self, raw_content: &str) -> Result<(PostMetadata, String)> {
        let raw_content = raw_content.trim_start();

        if !raw_content.starts_with("---\n") {
            return Err(anyhow!("metadata undetected."));
        }
        // 4 is the length of "---\n"
        // raw_content[4..].find() skipped the first four chars, so the index that find() returned should be added 4, makes
        // the index is the offset relative to raw_content[0].
        let end = 4 + raw_content[4..].find("\n---\n").ok_or_else(|| anyhow!("metadata end is not found."))?;
        let header = &raw_content[4..end];
        let body = &raw_content[end+9..];

        let metadata = self.parse_header(header)?;
        Ok((metadata, body.to_string()))
    }

    pub fn parse(mut self) -> Result<Self> {
        if self.content.is_none() {
            panic!("read() function should be called before build().");
        }

        let raw_content = self.content.clone().unwrap();  // Avoid clone()
        let (metadata, content) = self.split_markdown(&raw_content)?;
        self.metadata = Some(metadata);
        self.content = Some(content);
        Ok(self)
    }

    pub fn build(self) -> Post {
        let Self { reference, metadata, content } = self;

        if metadata.is_none() {
            panic!("parse() function should be called before build().");
        }
        let metadata = metadata.unwrap();
        let content = content.unwrap();

        Post {
            reference,
            metadata,
            content,
        }
    }
}
