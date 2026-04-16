use crate::env::template;
use crate::post::SharedPost;
use crate::render::Render;
use crate::site::SiteConf;
use pulldown_cmark::{html, CowStr, Event, Options, Parser};
use serde::Serialize;
use std::ops::Range;

pub struct PostRenderer<'a> {
    site: &'a SiteConf,
    post: &'a SharedPost,
}

impl<'a> PostRenderer<'a> {
    pub fn new(site: &'a SiteConf, post: &'a SharedPost) -> Self {
        Self { site, post }
    }

    fn render_markdown(&self) -> String {
        let (markdown_text, latex) = {
            let post = self.post.0.borrow();
            (post.body, post.metadata.latex)
        };
        render_markdown_content(markdown_text, latex)
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

fn render_markdown_content(markdown_text: &str, latex: bool) -> String {
    let mut options = Options::all();
    if !latex {
        options.remove(Options::ENABLE_MATH);
    }
    let parser = Parser::new_ext(markdown_text, options);
    let events = if latex {
        render_events_with_latex(parser.into_offset_iter(), markdown_text)
    } else {
        parser.collect()
    };

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());
    html_output
}

fn render_events_with_latex<'a>(
    parser: impl Iterator<Item = (Event<'a>, Range<usize>)>,
    markdown_text: &'a str,
) -> Vec<Event<'a>> {
    let mut rendered = Vec::new();
    let mut text_run = Vec::new();

    for (event, range) in parser {
        match event {
            Event::Text(_) | Event::SoftBreak | Event::HardBreak => text_run.push((event, range)),
            Event::InlineMath(text) => {
                rendered.extend(render_text_run_with_latex(
                    std::mem::take(&mut text_run),
                    markdown_text,
                ));
                push_math_event(&mut rendered, text.into_string(), false);
            }
            Event::DisplayMath(text) => {
                rendered.extend(render_text_run_with_latex(
                    std::mem::take(&mut text_run),
                    markdown_text,
                ));
                push_math_event(&mut rendered, text.into_string(), true);
            }
            other => {
                rendered.extend(render_text_run_with_latex(
                    std::mem::take(&mut text_run),
                    markdown_text,
                ));
                rendered.push(other);
            }
        }
    }

    rendered.extend(render_text_run_with_latex(text_run, markdown_text));
    rendered
}

fn render_text_run_with_latex<'a>(
    events: Vec<(Event<'a>, Range<usize>)>,
    markdown_text: &'a str,
) -> Vec<Event<'a>> {
    if events.is_empty() {
        return Vec::new();
    }

    let run_start = events
        .first()
        .map(|(_, range)| range.start)
        .expect("text run must have at least one event");
    let run_end = events
        .last()
        .map(|(_, range)| range.end)
        .expect("text run must have at least one event");
    let raw_run = &markdown_text[run_start..run_end];
    if !raw_run.contains('$') {
        return events.into_iter().map(|(event, _)| event).collect();
    }

    let mut rendered = Vec::new();
    let mut plain_text = String::new();
    let mut pending_math: Option<PendingMath> = None;
    let mut last_end = run_start;

    for (event, range) in events {
        if range.start > last_end {
            let skipped = &markdown_text[last_end..range.start];
            if skipped.chars().all(|ch| ch == '\\') {
                render_text_segment(skipped, &mut rendered, &mut plain_text, &mut pending_math);
            }
        }

        match event {
            Event::Text(_) => render_text_segment(
                &markdown_text[range.start..range.end],
                &mut rendered,
                &mut plain_text,
                &mut pending_math,
            ),
            Event::SoftBreak => {
                if let Some(pending) = pending_math.as_mut() {
                    pending.content.push('\n');
                } else {
                    push_text_event(&mut rendered, &mut plain_text);
                    rendered.push(Event::SoftBreak);
                }
            }
            Event::HardBreak => {
                if let Some(pending) = pending_math.as_mut() {
                    pending.content.push('\n');
                } else {
                    push_text_event(&mut rendered, &mut plain_text);
                    rendered.push(Event::HardBreak);
                }
            }
            _ => {}
        }

        last_end = range.end;
    }

    if let Some(pending) = pending_math {
        plain_text.push_str(if pending.display_mode { "$$" } else { "$" });
        plain_text.push_str(&pending.content);
    }

    push_text_event(&mut rendered, &mut plain_text);
    rendered
}

#[derive(Debug)]
struct PendingMath {
    content: String,
    display_mode: bool,
}

fn render_text_segment<'a>(
    text: &str,
    rendered: &mut Vec<Event<'a>>,
    plain_text: &mut String,
    pending_math: &mut Option<PendingMath>,
) {
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if let Some(display_mode) = pending_math.as_ref().map(|pending| pending.display_mode) {
            let closes_display = display_mode
                && index + 1 < chars.len()
                && chars[index] == '$'
                && chars[index + 1] == '$'
                && !is_escaped(&chars, index);
            let closes_inline = !display_mode && chars[index] == '$' && !is_escaped(&chars, index);

            if closes_display || closes_inline {
                let pending = pending_math
                    .take()
                    .expect("pending_math must exist when closing a latex fragment");
                push_math_event(rendered, pending.content, pending.display_mode);
                index += if display_mode { 2 } else { 1 };
                continue;
            }

            if let Some(pending) = pending_math.as_mut() {
                pending.content.push(chars[index]);
            }
            index += 1;
            continue;
        }

        if chars[index] == '$' && !is_escaped(&chars, index) {
            let display_mode = index + 1 < chars.len() && chars[index + 1] == '$';
            push_text_event(rendered, plain_text);
            *pending_math = Some(PendingMath {
                content: String::new(),
                display_mode,
            });
            index += if display_mode { 2 } else { 1 };
            continue;
        }

        plain_text.push(chars[index]);
        index += 1;
    }
}

fn push_text_event<'a>(rendered: &mut Vec<Event<'a>>, plain_text: &mut String) {
    if plain_text.is_empty() {
        return;
    }

    rendered.push(Event::Text(CowStr::Boxed(
        std::mem::take(plain_text).into_boxed_str(),
    )));
}

fn push_math_event<'a>(rendered: &mut Vec<Event<'a>>, content: String, display_mode: bool) {
    let display_mode = if display_mode { "true" } else { "false" };
    let html = format!(
        "<span class=\"arithmatex\" data-display-mode=\"{display_mode}\">{}</span>",
        escape_html(&content)
    );
    rendered.push(Event::Html(CowStr::Boxed(html.into_boxed_str())));
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

    #[test]
    fn renders_multiline_display_latex_fragments_for_katex() {
        let html = render_markdown_content("$$\nx^2 + y^2 = z^2\n$$", true);

        assert!(html.contains("class=\"arithmatex\""));
        assert!(html.contains("data-display-mode=\"true\""));
        assert!(html.contains("x^2 + y^2 = z^2"));
        assert!(!html.contains("$$"));
    }

    #[test]
    fn preserves_tex_escape_sequences_in_display_latex() {
        let html = render_markdown_content(
            "$$\n\\left\\{\n\\begin{aligned}\n-1, & x < 0 \\\\\n0, & x = 0\n\\end{aligned}\n\\right.\n$$",
            true,
        );

        assert!(html.contains("\\left\\{"));
        assert!(html.contains("\\begin{aligned}"));
        assert!(html.contains("\\\\"));
    }

    #[test]
    fn does_not_capture_blockquote_markers_inside_math() {
        let html = render_markdown_content(
            "> $f(x) = \n> \\left\\{\n> \\begin{aligned}\n> 1, & x \\leq 1 \\\\\n> 0, & x > 1\n> \\end{aligned}\n> \\right.$",
            true,
        );

        assert!(html.contains("\\left\\{"));
        assert!(!html.contains("&gt; \\left\\{"));
        assert!(!html.contains("&gt; \\begin{aligned}"));
    }

    #[test]
    fn does_not_render_math_without_latex_flag() {
        let html = render_markdown_content("Euler: $e^{i\\pi}+1=0$.", false);

        assert!(!html.contains("arithmatex"));
        assert!(!html.contains("math-inline"));
        assert!(html.contains("$e^{i\\pi}+1=0$."));
    }
}
