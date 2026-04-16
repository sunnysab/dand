use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};

use discover::PostWalker;

use crate::discover::WalkDirectory;
use crate::env::Environment;
use crate::index::{
    CategoryOverviewRenderer, CategoryRenderer, IndexRenderer, TagOverviewRenderer, TagRenderer,
};
use crate::post::PostRenderer;
use crate::render::Render;
use crate::site::SiteConf;

mod build;
mod datetime;
mod discover;
mod env;
mod index;
mod post;
mod render;
mod site;
mod taxonomy;

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

        prepare_output_dir(&path, &output_dir)?;
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
            let renderer = PostRenderer::new(&self.site, post);
            let html = renderer.html()?;
            let (relative_name, full_path) = {
                let post = post.0.borrow();
                (
                    post.reference.relative_name.clone(),
                    post.reference.full_path.clone(),
                )
            };

            let mut post_dir = self.output_dir.clone();
            post_dir.push(&relative_name);
            std::fs::create_dir_all(&post_dir).with_context(|| {
                format!("failed to create post directory: {}", post_dir.display())
            })?;

            let html_file = post_dir.join("index.html");
            std::fs::write(&html_file, html)
                .with_context(|| format!("failed to write post html: {}", html_file.display()))?;
            copy_post_resources(&full_path, &post_dir)?;
        }

        for (_, tag) in self.env.get_tags().iter() {
            let renderer = TagRenderer::new(&self.site, tag);
            let html = renderer.html()?;
            let tag_name = tag.slug();
            let tag_dir = self.output_dir.join("tags").join(&tag_name);
            std::fs::create_dir_all(&tag_dir).with_context(|| {
                format!("failed to create tag directory: {}", tag_dir.display())
            })?;

            let html_file = tag_dir.join("index.html");
            std::fs::write(&html_file, html)
                .with_context(|| format!("failed to write tag html: {}", html_file.display()))?;
        }

        for (_, category) in self.env.get_categories().iter() {
            let renderer = CategoryRenderer::new(&self.site, category);
            let html = renderer.html()?;
            let category_name = category.slug();
            let category_dir = self.output_dir.join("categories").join(&category_name);
            std::fs::create_dir_all(&category_dir).with_context(|| {
                format!(
                    "failed to create category directory: {}",
                    category_dir.display()
                )
            })?;

            let html_file = category_dir.join("index.html");
            std::fs::write(&html_file, html).with_context(|| {
                format!("failed to write category html: {}", html_file.display())
            })?;
        }

        let tags_overview = TagOverviewRenderer::new(&self.site, &self.env).html()?;
        let tags_index = self.output_dir.join("tags").join("index.html");
        if let Some(parent) = tags_index.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create tags directory: {}", parent.display())
            })?;
        }
        std::fs::write(&tags_index, tags_overview)
            .with_context(|| format!("failed to write tags overview: {}", tags_index.display()))?;

        let categories_overview = CategoryOverviewRenderer::new(&self.site, &self.env).html()?;
        let categories_index = self.output_dir.join("categories").join("index.html");
        if let Some(parent) = categories_index.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create categories directory: {}",
                    parent.display()
                )
            })?;
        }
        std::fs::write(&categories_index, categories_overview).with_context(|| {
            format!(
                "failed to write categories overview: {}",
                categories_index.display()
            )
        })?;
        Ok(())
    }

    fn copy_assets(&self) -> Result<()> {
        let src_dir = resolve_asset_dir(&self.path).context("failed to resolve asset directory")?;
        let dst_dir = self.output_dir.join("asset");
        copy_dir(&src_dir, &dst_dir)?;
        copy_root_static_files(&self.path, &self.output_dir)
    }

    fn render(&self) -> Result<()> {
        self.copy_assets()?;
        self.render_html()?;
        Ok(())
    }
}

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

fn prepare_output_dir(source_dir: &Path, output_dir: &Path) -> Result<()> {
    let source_dir = source_dir.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize source directory: {}",
            source_dir.display()
        )
    })?;
    let output_dir_existing = output_dir.exists();
    let output_dir_canonical = if output_dir_existing {
        Some(output_dir.canonicalize().with_context(|| {
            format!(
                "failed to canonicalize output directory: {}",
                output_dir.display()
            )
        })?)
    } else {
        None
    };

    if output_dir_canonical.as_deref() == Some(source_dir.as_path()) {
        bail!("output directory must be different from source directory");
    }

    if output_dir_existing {
        std::fs::remove_dir_all(output_dir).with_context(|| {
            format!("failed to clean output directory: {}", output_dir.display())
        })?;
    }
    std::fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create output directory: {}",
            output_dir.display()
        )
    })?;
    Ok(())
}

fn copy_root_static_files(source_dir: &Path, output_dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() {
            continue;
        }

        if path.extension() == Some("md".as_ref()) || path.file_name() == Some("dand.toml".as_ref())
        {
            continue;
        }

        std::fs::copy(&path, output_dir.join(entry.file_name()))?;
    }
    Ok(())
}

fn copy_post_resources(post_path: &Path, output_dir: &Path) -> Result<()> {
    let Some(parent) = post_path.parent() else {
        return Ok(());
    };
    let Some(stem) = post_path.file_stem() else {
        return Ok(());
    };

    let resource_dir = parent.join(stem);
    if resource_dir.is_dir() {
        copy_dir(&resource_dir, &output_dir.join(stem))?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build {
            source_dir,
            output_dir,
            config,
        } => {
            let config_path = resolve_config_path(&source_dir, config)?;
            let template_dir = resolve_template_dir(&source_dir)?;
            let report = build::build_site(&source_dir, &output_dir, &config_path, &template_dir)?;
            report.print_summary();
        }
        Commands::Serve {
            source_dir,
            output_dir,
            port,
            watch,
            interval_ms,
            config,
        } => run_preview(source_dir, output_dir, port, watch, interval_ms, config)?,
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
    /// Build the site, then start a local HTTP server for preview.
    Serve {
        /// Source directory containing posts and dand.toml
        source_dir: PathBuf,
        /// Output directory for generated HTML
        #[arg(short = 'o', long = "output")]
        output_dir: PathBuf,
        /// Local preview port
        #[arg(short = 'p', long = "port", default_value_t = 8000)]
        port: u16,
        /// Rebuild automatically when source/template files change
        #[arg(long = "watch")]
        watch: bool,
        /// Poll interval in milliseconds when --watch is enabled
        #[arg(long = "interval", default_value_t = 1000)]
        interval_ms: u64,
        /// Optional path to dand.toml
        #[arg(short = 'c', long = "config")]
        config: Option<PathBuf>,
    },
}

fn run_preview(
    source_dir: PathBuf,
    output_dir: PathBuf,
    port: u16,
    watch: bool,
    interval_ms: u64,
    config: Option<PathBuf>,
) -> Result<()> {
    let config_path = resolve_config_path(&source_dir, config)?;
    let template_dir = resolve_template_dir(&source_dir)?;

    let report = build::build_site(&source_dir, &output_dir, &config_path, &template_dir)?;
    report.print_summary();

    let address = format!("127.0.0.1:{port}");
    if !watch {
        eprintln!("Serving {}", address);
        return serve(&output_dir, &address);
    }

    let listener = bind_server(&address)?;
    let server_root = output_dir.clone();
    thread::spawn(move || {
        if let Err(error) = serve_with_listener(listener, server_root) {
            eprintln!("preview server stopped: {error:#}");
        }
    });

    let watch_paths = watch_paths(&source_dir, &config_path)?;
    let mut current_state = capture_watch_state(&watch_paths)?;
    let poll_interval = Duration::from_millis(interval_ms);
    eprintln!("Watching {} and serving {}", source_dir.display(), address);

    loop {
        thread::sleep(poll_interval);
        let observed_state = capture_watch_state(&watch_paths)?;
        let changed_paths = changed_watch_paths(&current_state, &observed_state);
        if changed_paths.is_empty() {
            continue;
        }

        current_state = observed_state;
        let changed_list = changed_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        eprintln!("Change detected: {changed_list}");
        eprintln!("Rebuilding...");
        match build::build_site(&source_dir, &output_dir, &config_path, &template_dir) {
            Ok(report) => report.print_summary(),
            Err(error) => eprintln!("Rebuild failed: {error:#}"),
        }
    }
}

fn build_blog(source_dir: &Path, output_dir: &Path, config: Option<PathBuf>) -> Result<Blog> {
    let conf_path = resolve_config_path(source_dir, config)?;
    BlogBuilder::new()
        .src_dir(source_dir)?
        .output_dir(output_dir)?
        .conf(&conf_path)?
        .build()
}

fn resolve_template_dir(source_dir: &Path) -> Result<PathBuf> {
    let mut candidates = vec![source_dir.join("template")];

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("template"));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("template"));
        }
    }

    if let Some(found) = candidates.into_iter().find(|p| p.is_dir()) {
        return Ok(found);
    }

    bail!("could not find template directory")
}

fn resolve_asset_dir(source_dir: &Path) -> Result<PathBuf> {
    let mut candidates = vec![source_dir.join("template").join("asset")];

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("template").join("asset"));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("template").join("asset"));
        }
    }

    if let Some(found) = candidates.into_iter().find(|path| path.is_dir()) {
        return Ok(found);
    }

    bail!("could not find template/asset directory")
}

fn resolve_config_path(source_dir: &Path, config: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = config {
        return Ok(path);
    }

    let candidate = source_dir.join("dand.toml");
    if candidate.is_file() {
        return Ok(candidate);
    }

    bail!("could not find dand.toml in {}", source_dir.display())
}

fn bind_server(address: &str) -> Result<TcpListener> {
    TcpListener::bind(address)
        .with_context(|| format!("failed to bind preview server to {}", address))
}

fn serve(root_dir: &Path, address: &str) -> Result<()> {
    let listener = bind_server(address)?;
    serve_with_listener(listener, root_dir.to_path_buf())
}

fn serve_with_listener(listener: TcpListener, root_dir: PathBuf) -> Result<()> {
    for stream in listener.incoming() {
        let stream = stream?;
        if let Err(error) = serve_connection(stream, &root_dir) {
            eprintln!("failed to serve request: {error:#}");
        }
    }
    Ok(())
}

fn serve_connection(mut stream: TcpStream, root_dir: &Path) -> Result<()> {
    let request_path = read_request_path(&stream)?;
    let response = match resolve_request_path(root_dir, &request_path) {
        Some(path) if path.is_file() => {
            let body = std::fs::read(&path)
                .with_context(|| format!("failed to read response file: {}", path.display()))?;
            build_response("200 OK", content_type(&path), &body)
        }
        _ => build_not_found_response(root_dir)?,
    };

    stream.write_all(&response)?;
    stream.flush()?;
    Ok(())
}

fn build_not_found_response(root_dir: &Path) -> Result<Vec<u8>> {
    let not_found_page = root_dir.join("404").join("index.html");
    if not_found_page.is_file() {
        let body = std::fs::read(&not_found_page)
            .with_context(|| format!("failed to read 404 page: {}", not_found_page.display()))?;
        return Ok(build_response(
            "404 Not Found",
            content_type(&not_found_page),
            &body,
        ));
    }

    Ok(build_response(
        "404 Not Found",
        "text/plain; charset=utf-8",
        b"Not Found",
    ))
}

fn read_request_path(stream: &TcpStream) -> Result<String> {
    let mut first_line = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut first_line)?;
    let mut parts = first_line.split_whitespace();
    let _method = parts
        .next()
        .ok_or_else(|| anyhow!("missing request method"))?;
    let path = parts
        .next()
        .ok_or_else(|| anyhow!("missing request path"))?
        .to_string();
    Ok(path)
}

fn resolve_request_path(root_dir: &Path, request_path: &str) -> Option<PathBuf> {
    let path_without_query = request_path.split(['?', '#']).next()?;
    let decoded_path = percent_decode(path_without_query)?;
    let relative = decoded_path.trim_start_matches('/');

    let mut clean = PathBuf::new();
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) | Component::ParentDir => return None,
        }
    }

    let candidate = root_dir.join(&clean);
    if relative.is_empty() || candidate.is_dir() {
        return Some(candidate.join("index.html"));
    }
    if candidate.is_file() {
        return Some(candidate);
    }

    let as_directory = root_dir.join(&clean).join("index.html");
    if as_directory.is_file() {
        return Some(as_directory);
    }
    None
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut index = 0;
    let mut decoded = Vec::with_capacity(bytes.len());

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hi = *bytes.get(index + 1)?;
            let lo = *bytes.get(index + 2)?;
            let hex = [hi, lo];
            let hex = std::str::from_utf8(&hex).ok()?;
            let value = u8::from_str_radix(hex, 16).ok()?;
            decoded.push(value);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).ok()
}

fn build_response(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let headers = format!(
        "HTTP/1.1 {status}\r\n\
Content-Type: {content_type}\r\n\
Content-Length: {}\r\n\
Cache-Control: no-store, no-cache, must-revalidate, max-age=0\r\n\
Pragma: no-cache\r\n\
Expires: 0\r\n\
Connection: close\r\n\
\r\n",
        body.len()
    );

    let mut response = headers.into_bytes();
    response.extend_from_slice(body);
    response
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

fn watch_paths(source_dir: &Path, config_path: &Path) -> Result<Vec<PathBuf>> {
    let mut roots = BTreeSet::new();
    roots.insert(source_dir.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize source directory: {}",
            source_dir.display()
        )
    })?);
    roots.insert(config_path.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize config path: {}",
            config_path.display()
        )
    })?);

    if let Ok(cwd) = std::env::current_dir() {
        let template_dir = cwd.join("template");
        if template_dir.exists() {
            roots.insert(template_dir.canonicalize().with_context(|| {
                format!(
                    "failed to canonicalize template directory: {}",
                    template_dir.display()
                )
            })?);
        }
    }

    Ok(roots.into_iter().collect())
}

fn capture_watch_state(paths: &[PathBuf]) -> Result<BTreeMap<PathBuf, u128>> {
    let mut state = BTreeMap::new();
    for path in paths {
        capture_watch_entry(path, &mut state)?;
    }
    Ok(state)
}

fn changed_watch_paths(
    before: &BTreeMap<PathBuf, u128>,
    after: &BTreeMap<PathBuf, u128>,
) -> Vec<PathBuf> {
    let mut changed = BTreeSet::new();

    for (path, timestamp) in after {
        match before.get(path) {
            Some(previous) if previous == timestamp => {}
            _ => {
                changed.insert(path.clone());
            }
        }
    }

    for path in before.keys() {
        if !after.contains_key(path) {
            changed.insert(path.clone());
        }
    }

    changed.into_iter().collect()
}

fn capture_watch_entry(path: &Path, state: &mut BTreeMap<PathBuf, u128>) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("failed to stat watched path: {}", path.display()))?;
    state.insert(path.to_path_buf(), modified_millis(metadata.modified()?));

    if metadata.is_dir() {
        for entry in std::fs::read_dir(path)
            .with_context(|| format!("failed to read watched directory: {}", path.display()))?
        {
            let entry = entry?;
            capture_watch_entry(&entry.path(), state)?;
        }
    }
    Ok(())
}

fn modified_millis(time: SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("dand-{prefix}-{timestamp}-{id}"))
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn build_blog_from_posts(posts: &[(&str, &str)]) -> (Blog, PathBuf, PathBuf) {
        let source_dir = unique_temp_dir("render-source");
        let output_dir = unique_temp_dir("render-output");
        let mut env = Environment::new();
        let walker = PostWalker::from(source_dir.clone());

        write_file(&source_dir.join("template/asset/css/main.css"), "body {}");

        for (relative_path, content) in posts {
            let full_path = source_dir.join(relative_path);
            write_file(&full_path, content);

            let reference = walker.open(full_path).unwrap();
            post::load(reference).unwrap().build(&mut env);
        }

        let blog = Blog {
            site: SiteConf {
                title: String::from("test"),
                home: String::from("https://example.com/"),
                description: String::from("test"),
                footer: site::FooterConf::default(),
            },
            path: source_dir.clone(),
            output_dir: output_dir.clone(),
            env,
        };

        (blog, source_dir, output_dir)
    }

    #[test]
    fn copy_assets_also_copies_root_static_files() {
        let source_dir = unique_temp_dir("root-static-source");
        let output_dir = unique_temp_dir("root-static-output");

        write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
        write_file(&source_dir.join("robots.txt"), "User-agent: *\n");

        let blog = Blog {
            site: SiteConf {
                title: String::from("test"),
                home: String::from("https://example.com/"),
                description: String::from("test"),
                footer: site::FooterConf::default(),
            },
            path: source_dir.clone(),
            output_dir: output_dir.clone(),
            env: Environment::new(),
        };

        blog.copy_assets().unwrap();

        assert!(output_dir.join("asset/css/main.css").is_file());
        assert!(output_dir.join("robots.txt").is_file());

        fs::remove_dir_all(&source_dir).unwrap();
        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn render_copies_post_resource_directories() {
        let source_dir = unique_temp_dir("post-resource-source");
        let output_dir = unique_temp_dir("post-resource-output");
        let post_path = source_dir.join("_posts/example.md");

        write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
        write_file(
            &post_path,
            "---\n\
title: Example\n\
date: 2024-01-01 00:00\n\
---\n\
\n\
![img](example/image.png)\n",
        );
        write_file(&source_dir.join("_posts/example/image.png"), "fake-image");

        let reference = PostWalker::from(source_dir.clone())
            .open(post_path)
            .unwrap();
        let mut env = Environment::new();
        post::load(reference).unwrap().build(&mut env);

        let blog = Blog {
            site: SiteConf {
                title: String::from("test"),
                home: String::from("https://example.com/"),
                description: String::from("test"),
                footer: site::FooterConf::default(),
            },
            path: source_dir.clone(),
            output_dir: output_dir.clone(),
            env,
        };

        blog.render().unwrap();

        assert!(output_dir.join("example/index.html").is_file());
        assert!(output_dir.join("example/example/image.png").is_file());

        fs::remove_dir_all(&source_dir).unwrap();
        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn render_groups_identical_tags_in_tags_overview() {
        let (blog, source_dir, output_dir) = build_blog_from_posts(&[
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
                "_posts/hidden-only.md",
                "---\n\
title: Hidden\n\
date: 2024-01-03 00:00\n\
is_hidden: true\n\
tags:\n\
  - 隐藏标签\n\
---\n\
\n\
Hidden\n",
            ),
        ]);

        blog.render().unwrap();

        let html = fs::read_to_string(output_dir.join("tags/index.html")).unwrap();

        assert!(html.contains("href=\"/tags/新东方/\""));
        assert!(html.contains("href=\"/tags/新概念英语/\""));
        assert!(html.contains(" & "));
        assert!(html.contains("2 篇"));
        assert!(!html.contains("隐藏标签"));

        fs::remove_dir_all(&source_dir).unwrap();
        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn render_categories_overview_remains_single_linked_items() {
        let (blog, source_dir, output_dir) = build_blog_from_posts(&[(
            "_posts/post-1.md",
            "---\n\
title: Post 1\n\
date: 2024-01-01 00:00\n\
categories:\n\
  - 随笔\n\
---\n\
\n\
One\n",
        )]);

        blog.render().unwrap();

        let html = fs::read_to_string(output_dir.join("categories/index.html")).unwrap();

        assert!(html.contains("href=\"/categories/随笔/\""));
        assert!(html.contains(">随笔</a>"));
        assert!(html.contains("1 篇"));
        assert!(!html.contains("</a> & <a"));

        fs::remove_dir_all(&source_dir).unwrap();
        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn resolve_request_path_maps_directories_to_index_html() {
        let output_dir = unique_temp_dir("serve-resolve-output");
        write_file(&output_dir.join("index.html"), "home");
        write_file(&output_dir.join("tags/index.html"), "tags");

        let root_path = resolve_request_path(&output_dir, "/").unwrap();
        let tags_path = resolve_request_path(&output_dir, "/tags/").unwrap();

        assert_eq!(root_path, output_dir.join("index.html"));
        assert_eq!(tags_path, output_dir.join("tags/index.html"));

        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn resolve_request_path_decodes_percent_encoded_directories() {
        let output_dir = unique_temp_dir("serve-resolve-encoded-output");
        write_file(&output_dir.join("categories/随笔/index.html"), "notes");

        let encoded_path = resolve_request_path(&output_dir, "/categories/%E9%9A%8F%E7%AC%94/");

        assert_eq!(
            encoded_path,
            Some(output_dir.join("categories/随笔/index.html"))
        );

        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn serve_connection_sends_no_cache_headers() {
        let output_dir = unique_temp_dir("serve-header-output");
        write_file(&output_dir.join("index.html"), "home");

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let root = output_dir.clone();

        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            serve_connection(stream, &root).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();

        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server.join().unwrap();

        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("Cache-Control: no-store, no-cache, must-revalidate, max-age=0"));
        assert!(response.contains("Pragma: no-cache"));
        assert!(response.contains("Expires: 0"));

        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn serve_connection_uses_site_404_page_when_present() {
        let output_dir = unique_temp_dir("serve-404-output");
        write_file(
            &output_dir.join("404/index.html"),
            "<h1>Custom Not Found</h1>",
        );

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let root = output_dir.clone();

        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            serve_connection(stream, &root).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client
            .write_all(
                b"GET /missing-page HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            )
            .unwrap();

        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server.join().unwrap();

        assert!(response.contains("HTTP/1.1 404 Not Found"));
        assert!(response.contains("<h1>Custom Not Found</h1>"));

        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn capture_watch_state_changes_after_file_update() {
        let watched_dir = unique_temp_dir("watch-state-dir");
        let file_path = watched_dir.join("note.txt");
        write_file(&file_path, "first");

        let before = capture_watch_state(std::slice::from_ref(&watched_dir)).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        write_file(&file_path, "second");
        let after = capture_watch_state(std::slice::from_ref(&watched_dir)).unwrap();

        assert_ne!(before, after);

        fs::remove_dir_all(&watched_dir).unwrap();
    }

    #[test]
    fn changed_watch_paths_reports_modified_file() {
        let watched_dir = unique_temp_dir("watch-diff-dir");
        let file_path = watched_dir.join("note.txt");
        write_file(&file_path, "first");

        let before = capture_watch_state(std::slice::from_ref(&watched_dir)).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        write_file(&file_path, "second");
        let after = capture_watch_state(std::slice::from_ref(&watched_dir)).unwrap();

        let changed = changed_watch_paths(&before, &after);
        assert!(changed.iter().any(|path| path == &file_path));

        fs::remove_dir_all(&watched_dir).unwrap();
    }

    #[test]
    fn serve_command_parses_watch_flag() {
        let cli = Cli::parse_from([
            "dand",
            "serve",
            "content",
            "-o",
            "public",
            "--watch",
            "--interval",
            "250",
        ]);

        match cli.command {
            Commands::Serve {
                source_dir,
                output_dir,
                watch,
                interval_ms,
                ..
            } => {
                assert_eq!(source_dir, PathBuf::from("content"));
                assert_eq!(output_dir, PathBuf::from("public"));
                assert!(watch);
                assert_eq!(interval_ms, 250);
            }
            _ => panic!("expected serve command"),
        }
    }
}
