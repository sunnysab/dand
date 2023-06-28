use std::sync::OnceLock;

use anyhow::{Context, Result};

use crate::env::Environment;
use crate::post::SharedPost;
use crate::site::SiteConf;

use super::render::Render;

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
        let _posts = self.env.get_posts();

        let mut parameters = tera::Context::new();
        parameters.insert("site", &self.site);
        parameters.insert("posts", &self.sorted_posts());

        let html = template().render("page/index.html", &parameters)?;
        Ok(html)
    }
}
