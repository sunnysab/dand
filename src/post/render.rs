use pulldown_cmark::{html, Options, Parser};

pub struct PostRenderer<'a> {
    markdown_text: &'a str,
}

impl<'a> PostRenderer<'a> {
    pub fn new(markdown_text: &'a str) -> Self {
        Self { markdown_text }
    }

    pub fn html(&self) -> String {
        let options = Options::all();
        let parser = Parser::new_ext(self.markdown_text, options);

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}
