use std::collections::BTreeMap;

use anyhow::Result;
use chrono::Datelike;
use serde::Serialize;

use crate::env::{template, Environment};
use crate::post::category::SharedCategory;
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
        let posts = self.sorted_posts();
        let mut parameters = tera::Context::new();
        parameters.insert("site", &self.site);
        parameters.insert("footer", &self.site.footer_view());
        parameters.insert("posts", &posts);
        parameters.insert("year_groups", &build_year_groups(&posts));
        parameters.insert("page_eyebrow", "最近文章");
        parameters.insert("page_title", &self.site.title);
        parameters.insert("document_title", &self.site.title);
        parameters.insert("supports_year_grouping", &true);
        parameters.insert("group_by_year_default", &true);

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
        parameters.insert("footer", &self.site.footer_view());
        parameters.insert("posts", &self.sorted_posts());
        parameters.insert("category", &self.tag.name());
        parameters.insert("category_href", &format!("/tags/{}/", self.tag.slug()));
        parameters.insert("overview_href", "/tags/");
        parameters.insert("overview_title", "返回标签总览");
        parameters.insert("page_eyebrow", "标签");
        parameters.insert("page_title", &self.tag.name());
        parameters.insert(
            "document_title",
            &format!("{} - {}", self.tag.name(), self.site.title),
        );
        parameters.insert("supports_year_grouping", &false);

        let html = template().render("page/index.html", &parameters)?;
        Ok(html)
    }
}

pub struct CategoryRenderer<'a> {
    site: &'a SiteConf,
    category: &'a SharedCategory,
}

impl<'a> CategoryRenderer<'a> {
    pub fn new(site: &'a SiteConf, category: &'a SharedCategory) -> Self {
        Self { site, category }
    }

    fn sorted_posts(&self) -> Vec<SharedPost> {
        sorted_public_posts(self.category.related_posts())
    }
}

impl<'a> Render for CategoryRenderer<'a> {
    fn html(&self) -> Result<String> {
        let mut parameters = tera::Context::new();
        parameters.insert("site", &self.site);
        parameters.insert("footer", &self.site.footer_view());
        parameters.insert("posts", &self.sorted_posts());
        parameters.insert("category", &self.category.name());
        parameters.insert(
            "category_href",
            &format!("/categories/{}/", self.category.slug()),
        );
        parameters.insert("overview_href", "/categories/");
        parameters.insert("overview_title", "返回分类总览");
        parameters.insert("page_eyebrow", "分类");
        parameters.insert("page_title", &self.category.name());
        parameters.insert(
            "document_title",
            &format!("{} - {}", self.category.name(), self.site.title),
        );
        parameters.insert("supports_year_grouping", &false);

        let html = template().render("page/index.html", &parameters)?;
        Ok(html)
    }
}

#[derive(Debug, Serialize)]
struct YearGroup {
    year: i32,
    posts: Vec<SharedPost>,
}

#[derive(Debug, Serialize)]
struct CategoryOverviewItem {
    name: String,
    href: String,
    post_count: usize,
}

#[derive(Debug, Serialize)]
struct TagOverviewLink {
    name: String,
    href: String,
}

#[derive(Debug, Serialize)]
struct TagOverviewItem {
    links: Vec<TagOverviewLink>,
    post_count: usize,
}

pub struct TagOverviewRenderer<'a> {
    site: &'a SiteConf,
    env: &'a Environment,
}

impl<'a> TagOverviewRenderer<'a> {
    pub fn new(site: &'a SiteConf, env: &'a Environment) -> Self {
        Self { site, env }
    }
}

impl<'a> Render for TagOverviewRenderer<'a> {
    fn html(&self) -> Result<String> {
        let items = build_tag_overview_items(self.env);
        render_tag_overview(self.site, "标签", items)
    }
}

pub struct CategoryOverviewRenderer<'a> {
    site: &'a SiteConf,
    env: &'a Environment,
}

impl<'a> CategoryOverviewRenderer<'a> {
    pub fn new(site: &'a SiteConf, env: &'a Environment) -> Self {
        Self { site, env }
    }
}

impl<'a> Render for CategoryOverviewRenderer<'a> {
    fn html(&self) -> Result<String> {
        let items = build_category_overview_items(
            self.env
                .get_categories()
                .iter()
                .map(|(_, category)| {
                    (
                        category.name(),
                        format!("/categories/{}/", category.slug()),
                        category.related_posts(),
                    )
                })
                .collect(),
        );
        render_category_overview(self.site, "分类", items)
    }
}

fn build_tag_overview_items(env: &Environment) -> Vec<TagOverviewItem> {
    let mut grouped_items: BTreeMap<String, (Vec<TagOverviewLink>, usize)> = BTreeMap::new();

    for (_, tag) in env.get_tags().iter() {
        let public_posts = sorted_public_posts(tag.related_posts());
        if public_posts.is_empty() {
            continue;
        }

        let post_count = public_posts.len();
        let mut post_ids: Vec<_> = public_posts
            .iter()
            .map(|post| post.0.borrow().reference.relative_name.clone())
            .collect();
        post_ids.sort();
        let group_key = post_ids.join("\n");

        let entry = grouped_items
            .entry(group_key)
            .or_insert_with(|| (Vec::new(), post_count));
        entry.0.push(TagOverviewLink {
            name: tag.name(),
            href: format!("/tags/{}/", tag.slug()),
        });
    }

    let mut items: Vec<_> = grouped_items
        .into_values()
        .map(|(mut links, post_count)| {
            links.sort_by(|a, b| a.name.cmp(&b.name));
            TagOverviewItem { links, post_count }
        })
        .collect();

    items.sort_by(|a, b| {
        let a_first = a.links.first().map(|link| link.name.as_str()).unwrap_or("");
        let b_first = b.links.first().map(|link| link.name.as_str()).unwrap_or("");
        b.post_count
            .cmp(&a.post_count)
            .then_with(|| a_first.cmp(b_first))
    });
    items
}

fn build_year_groups(posts: &[SharedPost]) -> Vec<YearGroup> {
    let mut groups: Vec<YearGroup> = Vec::new();

    for post in posts {
        let year = post.0.borrow().metadata.publish_date.year();
        match groups.last_mut() {
            Some(group) if group.year == year => group.posts.push(post.clone()),
            _ => groups.push(YearGroup {
                year,
                posts: vec![post.clone()],
            }),
        }
    }

    groups
}

fn render_tag_overview(
    site: &SiteConf,
    title: &str,
    items: Vec<TagOverviewItem>,
) -> Result<String> {
    let mut parameters = tera::Context::new();
    parameters.insert("site", site);
    parameters.insert("footer", &site.footer_view());
    parameters.insert("title", title);
    parameters.insert("items", &items);

    let html = template().render("page/tag_archive_list.html", &parameters)?;
    Ok(html)
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

fn public_post_count(posts: Vec<SharedPost>) -> usize {
    sorted_public_posts(posts).len()
}

fn build_category_overview_items(
    entries: Vec<(String, String, Vec<SharedPost>)>,
) -> Vec<CategoryOverviewItem> {
    let mut items: Vec<_> = entries
        .into_iter()
        .filter_map(|(name, href, posts)| {
            let post_count = public_post_count(posts);
            if post_count == 0 {
                return None;
            }
            Some(CategoryOverviewItem {
                name,
                href,
                post_count,
            })
        })
        .collect();

    items.sort_by(|a, b| {
        b.post_count
            .cmp(&a.post_count)
            .then_with(|| a.name.cmp(&b.name))
    });
    items
}

fn render_category_overview(
    site: &SiteConf,
    title: &str,
    items: Vec<CategoryOverviewItem>,
) -> Result<String> {
    let mut parameters = tera::Context::new();
    parameters.insert("site", site);
    parameters.insert("footer", &site.footer_view());
    parameters.insert("title", title);
    parameters.insert("items", &items);

    let html = template()
        .render("page/category_archive_list.html", &parameters)?;
    Ok(html)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::discover::{PostWalker, WalkDirectory};
    use crate::env::Environment;
    use crate::post;

    use super::build_tag_overview_items;

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("dand-index-{prefix}-{timestamp}-{id}"))
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn build_env(posts: &[(&str, &str)]) -> Environment {
        let source_dir = unique_temp_dir("tag-overview");
        let mut env = Environment::new();
        let walker = PostWalker::from(source_dir.clone());

        for (relative_path, content) in posts {
            let full_path = source_dir.join(relative_path);
            write_file(&full_path, content);

            let reference = walker.open(full_path).unwrap();
            post::load(reference).unwrap().build(&mut env);
        }

        fs::remove_dir_all(source_dir).unwrap();
        env
    }

    #[test]
    fn groups_tags_with_identical_public_post_sets() {
        let env = build_env(&[
            (
                "_posts/post-1.md",
                "---\n\
title: Post 1\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - 新东方\n\
  - 新概念英语\n\
---\n\
\n\
One\n",
            ),
            (
                "_posts/post-2.md",
                "---\n\
title: Post 2\n\
date: 2024-01-02 00:00\n\
tags:\n\
  - 新东方\n\
  - 新概念英语\n\
---\n\
\n\
Two\n",
            ),
            (
                "_drafts/post-3.md",
                "---\n\
title: Draft 3\n\
date: 2024-01-03 00:00\n\
tags:\n\
  - 新东方\n\
---\n\
\n\
Three\n",
            ),
            (
                "_posts/hidden-only.md",
                "---\n\
title: Hidden\n\
date: 2024-01-04 00:00\n\
is_hidden: true\n\
tags:\n\
  - 隐藏标签\n\
---\n\
\n\
Hidden\n",
            ),
        ]);

        let items = build_tag_overview_items(&env);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].post_count, 2);
        assert_eq!(items[0].links.len(), 2);
        assert_eq!(items[0].links[0].name, "新东方");
        assert_eq!(items[0].links[0].href, "/tags/新东方/");
        assert_eq!(items[0].links[1].name, "新概念英语");
        assert_eq!(items[0].links[1].href, "/tags/新概念英语/");
    }

    #[test]
    fn omits_tag_groups_without_public_posts() {
        let env = build_env(&[(
            "_posts/hidden-only.md",
            "---\n\
title: Hidden\n\
date: 2024-01-04 00:00\n\
is_hidden: true\n\
tags:\n\
  - 隐藏标签\n\
---\n\
\n\
Hidden\n",
        )]);

        let items = build_tag_overview_items(&env);

        assert!(items.is_empty());
    }
}
