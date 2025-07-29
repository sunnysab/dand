use anyhow::Result;

use crate::env::{template, Environment};
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
        let mut sorted_posts = Clone::clone(self.env.get_posts());

        sorted_posts = sorted_posts
            .into_iter()
            .filter(|p| !p.0.borrow().metadata.flag.is_hidden())
            .collect();
        sorted_posts.sort_by(|a, b| {
            b.0.borrow()
                .metadata
                .publish_date
                .cmp(&a.0.borrow().metadata.publish_date)
        });
        sorted_posts
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
