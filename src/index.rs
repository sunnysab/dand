use anyhow::Result;

use crate::env::{template, Environment};
use crate::post::tag::SharedTag;
use crate::post::SharedPost;
use crate::site::SiteConf;

use super::render::Render;

pub struct IndexRenderer<'a> {
    site: &'a SiteConf,
    env: &'a Environment,
}

impl<'a> IndexRenderer<'a> {
    pub fn new(site: &'a SiteConf, env: &'a Environment) -> IndexRenderer<'a> {
        Self { site, env }
    }

    fn sorted_posts(&self) -> Vec<SharedPost> {
        sorted_public_posts(self.env.get_posts().clone())
    }
}

impl<'a> Render for IndexRenderer<'a> {
    fn html(&self) -> Result<String> {
        let mut parameters = tera::Context::new();
        parameters.insert("site", &self.site);
        parameters.insert("posts", &self.sorted_posts());

        let html = template().render("page/index.html", &parameters)?;
        Ok(html)
    }
}

pub struct TagRenderer<'a> {
    site: &'a SiteConf,
    tag: &'a SharedTag,
}

impl<'a> TagRenderer<'a> {
    pub fn new(site: &'a SiteConf, tag: &'a SharedTag) -> Self {
        Self { site, tag }
    }

    fn sorted_posts(&self) -> Vec<SharedPost> {
        sorted_public_posts(self.tag.related_posts())
    }
}

impl<'a> Render for TagRenderer<'a> {
    fn html(&self) -> Result<String> {
        let mut parameters = tera::Context::new();
        parameters.insert("site", &self.site);
        parameters.insert("posts", &self.sorted_posts());
        parameters.insert("category", &self.tag.name());

        let html = template().render("page/index.html", &parameters)?;
        Ok(html)
    }
}

fn sorted_public_posts(mut posts: Vec<SharedPost>) -> Vec<SharedPost> {
    posts.retain(|post| {
        let metadata = post.0.borrow();
        let flag = &metadata.metadata.flag;
        !flag.is_hidden() && !flag.is_draft()
    });
    posts.sort_by(|a, b| {
        b.0.borrow()
            .metadata
            .publish_date
            .cmp(&a.0.borrow().metadata.publish_date)
    });
    posts
}
