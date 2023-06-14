use std::rc::Rc;

use anyhow::{anyhow, Context, Result};

use crate::post::tag::SharedTag;

use super::{Post, PostFlag, PostMetadata, PostMetadataRaw, PostReference};

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
            tags,
            ..
        } = raw_metadata;
        let publish_date = publish_date.unwrap_or(self.reference.created);
        let modify_date = modify_date.unwrap_or(self.reference.modified);

        let shared_tags = tags.into_iter().map(|tag| SharedTag::new(&tag)).collect();
        let result = PostMetadata {
            title,
            publish_date,
            modify_date,
            flag,
            tags: shared_tags,
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
            raw_content: Rc::new(raw_content),
            metadata,
            body_start_pos: body_index,
        })
    }
}
