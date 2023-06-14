use anyhow::Result;

use crate::discover::{PostWalker, WalkDirectory};
use crate::post::tag::TagSet;
use crate::post::{PostParser, SharedPost};

pub struct Environment {
    tags: TagSet,
    posts: Vec<SharedPost>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            tags: TagSet::new(),
            posts: vec![],
        }
    }

    pub fn walk(mut self, walker: PostWalker) -> Result<Self> {
        let post_references = walker.walk()?;

        for pr in post_references {
            // Read each post
            let post = PostParser::open(pr)?.parse()?;
            let shared_post = SharedPost::from(post);

            let new_tags = &shared_post.0.borrow().metadata.tags;
            let tags = TagSet::from(new_tags.clone());
            self.tags.merge(&tags);
        }
        Ok(self)
    }
}
