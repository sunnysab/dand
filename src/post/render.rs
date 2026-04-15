use crate::env::template;
use crate::post::SharedPost;
use crate::render::Render;
use crate::site::SiteConf;
use pulldown_cmark::{html, CowStr, Event, Options, Parser};
use serde::Serialize;

pub struct PostRenderer<'a> {
    site: &'a SiteConf,
    post: &'a SharedPost,
}

impl<'a> PostRenderer<'a> {
    pub fn new(site: &'a SiteConf, post: &'a SharedPost) -> Self {
        Self { site, post }
    }

    fn render_markdown(&self) -> String {
        let (markdown_text, use_latex) = {
            let post = self.post.0.borrow();
            (post.body, post.metadata.use_latex)
        };
        render_markdown_content(markdown_text, use_latex)
    }
}

impl<'a> Render for PostRenderer<'a> {
    fn html(&self) -> anyhow::Result<String> {
        let mut parameters = tera::Context::new();
        parameters.insert("site", &self.site);
        parameters.insert("footer", &self.site.footer_view());
        parameters.insert("post", &self.post);
        let (tags, categories) = self.taxonomy_links();
        parameters.insert("tags", &tags);
        parameters.insert("categories", &categories);

        let content = self.render_markdown();
        parameters.insert("content", &content);

        let html = template().render("page/post.html", &parameters)?;
        Ok(html)
    }
}

#[derive(Debug, Serialize)]
struct TaxonomyLink {
    name: String,
    href: String,
}

impl<'a> PostRenderer<'a> {
    fn taxonomy_links(&self) -> (Vec<TaxonomyLink>, Vec<TaxonomyLink>) {
        let post = self.post.0.borrow();
        let tags = post
            .metadata
            .tags
            .iter()
            .map(|tag| {
                let name = tag.name();
                TaxonomyLink {
                    href: format!("/tags/{}/", tag.slug()),
                    name,
                }
            })
            .collect();
        let categories = post
            .metadata
            .categories
            .iter()
            .map(|category| {
                let name = category.name();
                TaxonomyLink {
                    href: format!("/categories/{}/", category.slug()),
                    name,
                }
            })
            .collect();
        (tags, categories)
    }
}

fn render_markdown_content(markdown_text: &str, use_latex: bool) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(markdown_text, options).flat_map(|event| {
        if !use_latex {
            return vec![event];
        }

        match event {
            Event::Text(text) => render_text_with_latex(text),
            other => vec![other],
        }
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn render_text_with_latex(text: CowStr<'_>) -> Vec<Event<'_>> {
    let rendered = render_latex_fragments(text.as_ref());
    if matches!(rendered.as_slice(), [LatexFragment::Text(_)]) {
        return vec![Event::Text(text)];
    }

    rendered
        .into_iter()
        .map(|fragment| match fragment {
            LatexFragment::Text(text) => Event::Text(CowStr::Boxed(text.into_boxed_str())),
            LatexFragment::Math { content, display_mode } => {
                let display_mode = if display_mode { "true" } else { "false" };
                let html = format!(
                    "<span class=\"arithmatex\" data-display-mode=\"{display_mode}\">{}</span>",
                    escape_html(&content)
                );
                Event::Html(CowStr::Boxed(html.into_boxed_str()))
            }
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
enum LatexFragment {
    Text(String),
    Math { content: String, display_mode: bool },
}

fn render_latex_fragments(text: &str) -> Vec<LatexFragment> {
    let chars: Vec<char> = text.chars().collect();
    let mut fragments = Vec::new();
    let mut plain = String::new();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] != '$' || is_escaped(&chars, index) {
            plain.push(chars[index]);
            index += 1;
            continue;
        }

        let display_mode = index + 1 < chars.len() && chars[index + 1] == '$';
        let delimiter_len = if display_mode { 2 } else { 1 };

        if let Some(end) = find_matching_delimiter(&chars, index + delimiter_len, display_mode) {
            if !plain.is_empty() {
                fragments.push(LatexFragment::Text(std::mem::take(&mut plain)));
            }

            let content: String = chars[index + delimiter_len..end].iter().collect();
            fragments.push(LatexFragment::Math {
                content,
                display_mode,
            });
            index = end + delimiter_len;
        } else {
            plain.push(chars[index]);
            index += 1;
        }
    }

    if !plain.is_empty() {
        fragments.push(LatexFragment::Text(plain));
    }

    if fragments.is_empty() {
        fragments.push(LatexFragment::Text(String::new()));
    }

    fragments
}

fn find_matching_delimiter(chars: &[char], mut index: usize, display_mode: bool) -> Option<usize> {
    while index < chars.len() {
        if chars[index] == '$' && !is_escaped(chars, index) {
            if display_mode {
                if index + 1 < chars.len() && chars[index + 1] == '$' {
                    return Some(index);
                }
            } else {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

fn is_escaped(chars: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }

    let mut slash_count = 0;
    let mut current = index;
    while current > 0 {
        current -= 1;
        if chars[current] != '\\' {
            break;
        }
        slash_count += 1;
    }
    slash_count % 2 == 1
}

fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::render_markdown_content;

    #[test]
    fn renders_inline_latex_fragments_for_katex() {
        let html = render_markdown_content("Euler: $e^{i\\pi}+1=0$.", true);

        assert!(html.contains("class=\"arithmatex\""));
        assert!(html.contains("data-display-mode=\"false\""));
        assert!(html.contains("e^{i\\pi}+1=0"));
    }

    #[test]
    fn renders_display_latex_fragments_for_katex() {
        let html = render_markdown_content("$$x^2 + y^2 = z^2$$", true);

        assert!(html.contains("class=\"arithmatex\""));
        assert!(html.contains("data-display-mode=\"true\""));
        assert!(html.contains("x^2 + y^2 = z^2"));
    }
}
