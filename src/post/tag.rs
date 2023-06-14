use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::post::SharedPost;

#[derive(Default)]
pub struct Tag {
    pub name: String,
    pub related_posts: Vec<SharedPost>,
}

#[derive(Clone)]
pub struct SharedTag(Rc<RefCell<Tag>>);

impl SharedTag {
    pub fn new(name: &str) -> Self {
        let new_tag = Tag {
            name: name.to_string(),
            related_posts: vec![],
        };
        Self(Rc::new(RefCell::new(new_tag)))
    }

    pub fn relate(&self, post: &SharedPost) {
        let mut tag = RefCell::borrow_mut(&self.0);
        tag.related_posts.push(post.clone());

        post.add_tag(self)
    }

    pub fn merge_with(&self, other: &Self) {
        let other = &other.0.borrow().related_posts;
        for post in other {
            self.relate(post);
        }
    }
}

pub struct TagSet {
    tags: HashMap<String, SharedTag>,
}

impl TagSet {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
        }
    }

    pub fn add(&mut self, tag: &str, post: &SharedPost) -> SharedTag {
        if let Some(existed_tag) = self.tags.get(tag) {
            existed_tag.relate(post);
            existed_tag.clone()
        } else {
            let shared_new_tag = SharedTag::new(tag);
            shared_new_tag.relate(post);
            self.tags.insert(tag.to_string(), shared_new_tag.clone());

            shared_new_tag
        }
    }

    pub fn merge(&mut self, other: &Self) {
        for (key, value) in &other.tags {
            if let Some(existed_tag) = self.tags.get(key) {
                existed_tag.merge_with(value);
            }
        }
    }
}

impl From<Vec<SharedTag>> for TagSet {
    fn from(tags: Vec<SharedTag>) -> Self {
        let tags = tags
            .into_iter()
            .map(|tag| {
                let name = tag.0.borrow().name.clone();
                (name, tag)
            })
            .collect();
        Self { tags }
    }
}
