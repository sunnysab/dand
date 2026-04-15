use anyhow::Context;
use serde::Serialize;
use std::sync::OnceLock;

use crate::post::category::CategorySet;
use crate::post::tag::TagSet;
use crate::post::SharedPost;

static SITE_TEMPLATE: OnceLock<tera::Tera> = OnceLock::new();

fn init_template() -> tera::Tera {
    let template_dir = "template/**/*.html";
    let mut tera = tera::Tera::new(template_dir)
        .with_context(|| format!("failed to load template from {}", template_dir))
        .unwrap();

    tera.autoescape_on(vec![]);
    tera
}

pub fn template() -> &'static tera::Tera {
    SITE_TEMPLATE.get_or_init(init_template)
}

#[derive(Debug, Serialize)]
pub struct Environment {
    categories: CategorySet,
    tags: TagSet,
    posts: Vec<SharedPost>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            categories: CategorySet::new(),
            tags: TagSet::new(),
            posts: vec![],
        }
    }

    pub fn get_categories_mut(&mut self) -> &mut CategorySet {
        &mut self.categories
    }

    pub fn get_categories(&self) -> &CategorySet {
        &self.categories
    }

    pub fn get_tags_mut(&mut self) -> &mut TagSet {
        &mut self.tags
    }

    pub fn get_tags(&self) -> &TagSet {
        &self.tags
    }

    pub fn get_posts(&self) -> &Vec<SharedPost> {
        &self.posts
    }

    pub fn add_post(&mut self, post: SharedPost) {
        self.posts.push(post);
    }
}
