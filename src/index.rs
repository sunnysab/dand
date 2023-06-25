use std::sync::OnceLock;

use anyhow::{Context, Result};

use crate::env::Environment;

use super::render::Render;

static SITE_TEMPLATE: OnceLock<tera::Tera> = OnceLock::new();

fn init_template() -> tera::Tera {
    let template_dir = "template/*";

    tera::Tera::new(template_dir)
        .with_context(|| format!("Failed to load template from {}", template_dir))
        .unwrap()
}

pub fn template() -> &'static tera::Tera {
    SITE_TEMPLATE.get_or_init(init_template)
}

pub struct IndexRenderer<'a> {
    env: &'a Environment,
}

impl<'a> IndexRenderer<'a> {
    pub fn new(env: &'a Environment) -> IndexRenderer<'a> {
        Self { env }
    }
}

impl<'a> Render for IndexRenderer<'a> {
    fn html(&self) -> Result<String> {
        let _posts = self.env.get_posts();

        let parameters = tera::Context::new();
        // parameters.insert("site", self.env);
        // parameters.insert("posts", &posts);

        let html = String::new();
        template().render("page/index.html", &parameters)?;
        Ok(html)
    }
}
