use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::post::SharedPost;

#[derive(Debug)]
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
}

impl Display for SharedTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = RefCell::borrow(&self.0);

        let name = tag.name.as_str();
        let posts = tag
            .related_posts
            .iter()
            .take(3)
            .fold(String::new(), |l, e| {
                l + &e.0.borrow().metadata.title + ", "
            });
        write!(f, "SharedTag ({name}) [{posts}...]")
    }
}

impl Debug for SharedTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
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
}
