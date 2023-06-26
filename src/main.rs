use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Result};

use discover::PostWalker;

use crate::discover::WalkDirectory;
use crate::env::Environment;
use crate::index::IndexRenderer;
use crate::render::Render;
use crate::site::SiteConf;

mod datetime;
mod discover;
mod env;
mod index;
mod post;
mod render;
mod site;

#[derive(Default)]
struct BlogBuilder {
    site: Option<SiteConf>,

    path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
}

impl BlogBuilder {
    fn new() -> Self {
        Default::default()
    }

    fn src_dir(self, path: &str) -> Result<Self> {
        let path = PathBuf::from_str(path)?;
        if !path.is_dir() {
            bail!("source directory must be a directory");
        } else if !path.exists() {
            bail!("source directory does not exist");
        }

        let path = Some(path);
        Ok(Self { path, ..self })
    }

    fn output_dir(self, path: &str) -> Result<Self> {
        let path = PathBuf::from_str(path)?;
        // Checks against this path will be deferred until build()
        // if !path.is_dir() {
        //     bail!("output directory must be a directory");
        // }

        let output_dir = Some(path);
        Ok(Self { output_dir, ..self })
    }

    fn conf(self, site_conf: &str) -> Result<Self> {
        let conf = SiteConf::load(site_conf)?;
        let site = Option::from(conf);
        Ok(Self { site, ..self })
    }

    fn build(self) -> Result<Blog> {
        let path = self
            .path
            .ok_or_else(|| anyhow!("src_dir() must be called before build()."))?;
        let output_dir = self
            .output_dir
            .ok_or_else(|| anyhow!("output_dir() must be called before build()."))?;
        let site = self
            .site
            .ok_or_else(|| anyhow!("site() must be called before build()."))?;

        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir).with_context(|| {
                let output_dir = output_dir.display();
                format!("failed to create output directory: {output_dir}")
            })?;
        }
        let mut env = Environment::new();
        let walker = PostWalker::from(path.clone());
        for post_ref in walker.walk()?.into_iter() {
            post::load(post_ref)?.build(&mut env)?;
        }

        Ok(Blog {
            site,
            path,
            output_dir,
            env,
        })
    }
}

struct Blog {
    site: SiteConf,
    path: PathBuf,
    output_dir: PathBuf,
    env: Environment,
}

impl Blog {
    fn render(&self) -> Result<()> {
        let index_render = IndexRenderer::new(&self.env);
        let index_html = index_render.html()?;

        println!("{index_html}");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let blog = BlogBuilder::new()
        .src_dir("/home/sunnysab/Blog/source/_posts/")?
        .output_dir("/home/sunnysab/Blog/output/")?
        .conf("/home/sunnysab/Blog/source/_posts/dand.toml")?
        .build()?;

    Ok(())
}
