use crate::env::template;
use crate::post::SharedPost;
use crate::render::Render;
use crate::site::SiteConf;
use pulldown_cmark::{html, Options, Parser};

pub struct PostRenderer<'a> {
    site: &'a SiteConf,
    post: &'a SharedPost,
}

impl<'a> PostRenderer<'a> {
    pub fn new(site: &'a SiteConf, post: &'a SharedPost) -> Self {
        Self { site, post }
    }

    fn render_markdown(&self) -> String {
        let options = Options::all();
        let markdown_text = self.post.0.borrow().body;
        let parser = Parser::new_ext(markdown_text, options);

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}

impl<'a> Render for PostRenderer<'a> {
    fn html(&self) -> anyhow::Result<String> {
        let mut parameters = tera::Context::new();
        parameters.insert("site", &self.site);
        parameters.insert("post", &self.post);

        let content = self.render_markdown();
        parameters.insert("content", &content);

        let html = template().render("page/post.html", &parameters)?;
        Ok(html)
    }
}
