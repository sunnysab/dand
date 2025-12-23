use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};

use discover::PostWalker;

use crate::discover::WalkDirectory;
use crate::env::Environment;
use crate::index::IndexRenderer;
use crate::post::PostRenderer;
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

    fn src_dir(self, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.is_dir() {
            bail!("source directory must be a directory");
        } else if !path.exists() {
            bail!("source directory does not exist");
        }

        let path = Some(path);
        Ok(Self { path, ..self })
    }

    fn output_dir(self, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        // Checks against this path will be deferred until build()
        // if !path.is_dir() {
        //     bail!("output directory must be a directory");
        // }

        let output_dir = Some(path);
        Ok(Self { output_dir, ..self })
    }

    fn conf(self, site_conf: impl AsRef<Path>) -> Result<Self> {
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
            let path = post_ref.full_path.display().to_string();

            post::load(post_ref)
                .with_context(|| format!("failed to load post: {}", path))?
                .build(&mut env);
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
    fn render_html(&self) -> Result<()> {
        let index_render = IndexRenderer::new(&self.site, &self.env);
        let index_html = index_render.html()?;

        let mut index_file = self.output_dir.clone();
        index_file.push("index.html");

        std::fs::write(&index_file, &index_html)?;

        for post in self.env.get_posts() {
            let renderer = PostRenderer::new(&self.site, &post);
            let html = renderer.html();
            let mut html_file = self.output_dir.clone();
            html_file.push(format!("{:?}.html", post.0.borrow().reference.full_path));
            std::fs::write(&html_file, html)?;
        }
        Ok(())
    }

    fn copy_assets(&self) -> Result<()> {
        // TODO: Copy assets when it necessary

        fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
            use std::fs;

            if !dst.exists() {
                fs::create_dir_all(dst)?;
            }
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let path = entry.path();
                let dst_path = dst.join(path.file_name().unwrap());

                if entry.file_type()?.is_dir() {
                    copy_dir(&path, &dst_path)?;
                } else {
                    fs::copy(&path, &dst_path)?;
                }
            }
            Ok(())
        }

        let src_dir = resolve_asset_dir().context("failed to resolve asset directory")?;
        let dst_dir = self.output_dir.join("asset");
        copy_dir(&src_dir, &dst_dir)?;
        Ok(())
    }

    fn render(&self) -> Result<()> {
        self.copy_assets()?;
        self.render_html()?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build {
            source_dir,
            output_dir,
            config,
        } => {
            let conf_path = match config {
                Some(path) => path,
                None => source_dir.join("dand.toml"),
            };
            let blog = BlogBuilder::new()
                .src_dir(&source_dir)?
                .output_dir(&output_dir)?
                .conf(&conf_path)?
                .build()?;
            blog.render()?;
        }
    }
    Ok(())
}

#[derive(Parser)]
#[command(name = "dand", version, about = "A minimal static site generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the site from a source directory.
    Build {
        /// Source directory containing posts and dand.toml
        source_dir: PathBuf,
        /// Output directory for generated HTML
        #[arg(short = 'o', long = "output")]
        output_dir: PathBuf,
        /// Optional path to dand.toml
        #[arg(short = 'c', long = "config")]
        config: Option<PathBuf>,
    },
}

fn resolve_asset_dir() -> Result<PathBuf> {
    let cwd_assets = PathBuf::from("template").join("asset");
    if cwd_assets.is_dir() {
        return Ok(cwd_assets);
    }

    let exe_path = std::env::current_exe().context("failed to resolve current executable")?;
    if let Some(exe_dir) = exe_path.parent() {
        let exe_assets = exe_dir.join("template").join("asset");
        if exe_assets.is_dir() {
            return Ok(exe_assets);
        }
    }

    bail!("could not find template/asset directory")
}
