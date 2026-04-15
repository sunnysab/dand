use anyhow::Result;
use serde::Serialize;

use crate::env::{template, Environment};
use crate::post::category::SharedCategory;
use crate::post::tag::SharedTag;
use crate::post::SharedPost;
use crate::site::SiteConf;
use crate::taxonomy::{slug_for, SlugMap};

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
    slugs: &'a SlugMap,
}

impl<'a> TagRenderer<'a> {
    pub fn new(site: &'a SiteConf, tag: &'a SharedTag, slugs: &'a SlugMap) -> Self {
        Self { site, tag, slugs }
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
        parameters.insert("category_href", &format!("/tags/{}/", slug_for(&self.tag.name(), self.slugs)));

        let html = template().render("page/index.html", &parameters)?;
        Ok(html)
    }
}

pub struct CategoryRenderer<'a> {
    site: &'a SiteConf,
    category: &'a SharedCategory,
    slugs: &'a SlugMap,
}

impl<'a> CategoryRenderer<'a> {
    pub fn new(site: &'a SiteConf, category: &'a SharedCategory, slugs: &'a SlugMap) -> Self {
        Self {
            site,
            category,
            slugs,
        }
    }

    fn sorted_posts(&self) -> Vec<SharedPost> {
        sorted_public_posts(self.category.related_posts())
    }
}

impl<'a> Render for CategoryRenderer<'a> {
    fn html(&self) -> Result<String> {
        let mut parameters = tera::Context::new();
        parameters.insert("site", &self.site);
        parameters.insert("posts", &self.sorted_posts());
        parameters.insert("category", &self.category.name());
        parameters.insert(
            "category_href",
            &format!(
                "/categories/{}/",
                slug_for(&self.category.name(), self.slugs)
            ),
        );

        let html = template().render("page/index.html", &parameters)?;
        Ok(html)
    }
}

#[derive(Debug, Serialize)]
struct ArchiveOverviewItem {
    name: String,
    href: String,
    post_count: usize,
}

pub struct TagOverviewRenderer<'a> {
    site: &'a SiteConf,
    env: &'a Environment,
    slugs: &'a SlugMap,
}

impl<'a> TagOverviewRenderer<'a> {
    pub fn new(site: &'a SiteConf, env: &'a Environment, slugs: &'a SlugMap) -> Self {
        Self { site, env, slugs }
    }
}

impl<'a> Render for TagOverviewRenderer<'a> {
    fn html(&self) -> Result<String> {
        let items = build_archive_items(
            self.env
                .get_tags()
                .iter()
                .map(|(_, tag)| {
                    let name = tag.name();
                    (
                        name.clone(),
                        format!("/tags/{}/", slug_for(&name, self.slugs)),
                        tag.related_posts(),
                    )
                })
                .collect(),
        );
        render_archive_overview(self.site, "标签", items)
    }
}

pub struct CategoryOverviewRenderer<'a> {
    site: &'a SiteConf,
    env: &'a Environment,
    slugs: &'a SlugMap,
}

impl<'a> CategoryOverviewRenderer<'a> {
    pub fn new(site: &'a SiteConf, env: &'a Environment, slugs: &'a SlugMap) -> Self {
        Self { site, env, slugs }
    }
}

impl<'a> Render for CategoryOverviewRenderer<'a> {
    fn html(&self) -> Result<String> {
        let items = build_archive_items(
            self.env
                .get_categories()
                .iter()
                .map(|(_, category)| {
                    let name = category.name();
                    (
                        name.clone(),
                        format!("/categories/{}/", slug_for(&name, self.slugs)),
                        category.related_posts(),
                    )
                })
                .collect(),
        );
        render_archive_overview(self.site, "分类", items)
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

fn public_post_count(posts: Vec<SharedPost>) -> usize {
    sorted_public_posts(posts).len()
}

fn build_archive_items(entries: Vec<(String, String, Vec<SharedPost>)>) -> Vec<ArchiveOverviewItem> {
    let mut items: Vec<_> = entries
        .into_iter()
        .filter_map(|(name, href, posts)| {
            let post_count = public_post_count(posts);
            if post_count == 0 {
                return None;
            }
            Some(ArchiveOverviewItem {
                name,
                href,
                post_count,
            })
        })
        .collect();

    items.sort_by(|a, b| a.name.cmp(&b.name));
    items
}

fn render_archive_overview(
    site: &SiteConf,
    title: &str,
    items: Vec<ArchiveOverviewItem>,
) -> Result<String> {
    let mut parameters = tera::Context::new();
    parameters.insert("site", site);
    parameters.insert("title", title);
    parameters.insert("items", &items);

    let html = template().render("page/archive_list.html", &parameters)?;
    Ok(html)
}
