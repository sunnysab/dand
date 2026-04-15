use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use serde::{Serialize, Serializer};

use crate::post::SharedPost;
use crate::taxonomy::{should_replace_display_name, slugify};

#[derive(Debug, Serialize)]
pub struct Category {
    pub slug: String,
    pub name: String,
    pub related_posts: Vec<SharedPost>,
}

#[derive(Clone)]
pub struct SharedCategory(Rc<RefCell<Category>>);

impl SharedCategory {
    pub fn new(name: &str) -> Self {
        let new_category = Category {
            slug: slugify(name),
            name: name.to_string(),
            related_posts: vec![],
        };
        Self(Rc::new(RefCell::new(new_category)))
    }

    pub fn relate(&self, post: &SharedPost) {
        {
            let mut category = RefCell::borrow_mut(&self.0);
            if !category
                .related_posts
                .iter()
                .any(|related| Rc::ptr_eq(&related.0, &post.0))
            {
                category.related_posts.push(post.clone());
            }
        }

        post.add_category(self)
    }

    pub fn slug(&self) -> String {
        self.0.borrow().slug.clone()
    }

    pub fn name(&self) -> String {
        self.0.borrow().name.clone()
    }

    pub fn related_posts(&self) -> Vec<SharedPost> {
        self.0.borrow().related_posts.clone()
    }

    pub fn update_display_name(&self, candidate: &str) {
        let mut category = self.0.borrow_mut();
        if should_replace_display_name(&category.name, candidate) {
            category.name = candidate.to_string();
        }
    }
}

impl Display for SharedCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let category = RefCell::borrow(&self.0);

        let name = category.name.as_str();
        let posts = category
            .related_posts
            .iter()
            .take(3)
            .fold(String::new(), |l, e| {
                l + &e.0.borrow().metadata.title + ", "
            });
        write!(f, "SharedCategory ({name}) [{posts}...]")
    }
}

impl Debug for SharedCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Serialize for SharedCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let inner = self.0.borrow();
        serializer.serialize_str(inner.name.as_str())
    }
}

#[derive(Debug, Serialize)]
pub struct CategorySet {
    categories: HashMap<String, SharedCategory>,
}

impl CategorySet {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }

    pub fn add(&mut self, category: &str, post: &SharedPost) -> SharedCategory {
        let slug = slugify(category);
        if let Some(existed_category) = self.categories.get(&slug) {
            existed_category.update_display_name(category);
            existed_category.relate(post);
            existed_category.clone()
        } else {
            let shared_new_category = SharedCategory::new(category);
            shared_new_category.relate(post);
            self.categories.insert(slug, shared_new_category.clone());

            shared_new_category
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &SharedCategory)> {
        self.categories.iter()
    }
}
