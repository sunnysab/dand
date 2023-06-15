use crate::post::tag::TagSet;
use crate::post::SharedPost;

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

    pub fn get_tags(&self) -> &TagSet {
        &self.tags
    }

    pub fn get_tags_mut(&mut self) -> &mut TagSet {
        &mut self.tags
    }

    pub fn get_posts(&self) -> &Vec<SharedPost> {
        &self.posts
    }

    pub fn add_post(&mut self, post: SharedPost) {
        self.posts.push(post);
    }

    pub fn register() {}
}
