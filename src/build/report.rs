#[derive(Debug, Default)]
pub struct BuildReport {
    pub is_incremental: bool,
    pub rendered_posts: usize,
    pub deleted_posts: usize,
    pub rendered_tags: usize,
    pub rendered_categories: usize,
    pub rendered_index: bool,
    pub rendered_tags_overview: bool,
    pub rendered_categories_overview: bool,
    pub copied_assets: bool,
    pub copied_root_statics: usize,
    pub deleted_static_files: usize,
}

impl BuildReport {
    pub fn print_summary(&self) {
        if self.is_incremental {
            if self.is_noop() {
                eprintln!("No changes detected, skipping rebuild.");
                return;
            }
            eprintln!("Incremental build complete.");
        } else {
            eprintln!("Full build complete.");
        }

        if self.rendered_posts > 0 {
            eprintln!("  Rendered {} post(s)", self.rendered_posts);
        }
        if self.deleted_posts > 0 {
            eprintln!("  Deleted {} post output(s)", self.deleted_posts);
        }
        if self.rendered_tags > 0 {
            eprintln!("  Rendered {} tag page(s)", self.rendered_tags);
        }
        if self.rendered_categories > 0 {
            eprintln!("  Rendered {} category page(s)", self.rendered_categories);
        }
        if self.rendered_index {
            eprintln!("  Rendered index");
        }
        if self.rendered_tags_overview {
            eprintln!("  Rendered tags overview");
        }
        if self.rendered_categories_overview {
            eprintln!("  Rendered categories overview");
        }
        if self.copied_assets {
            eprintln!("  Copied template assets");
        }
        if self.copied_root_statics > 0 {
            eprintln!("  Copied {} root static file(s)", self.copied_root_statics);
        }
        if self.deleted_static_files > 0 {
            eprintln!("  Deleted {} static file(s)", self.deleted_static_files);
        }
    }

    pub fn is_noop(&self) -> bool {
        self.rendered_posts == 0
            && self.deleted_posts == 0
            && self.rendered_tags == 0
            && self.rendered_categories == 0
            && !self.rendered_index
            && !self.rendered_tags_overview
            && !self.rendered_categories_overview
            && !self.copied_assets
            && self.copied_root_statics == 0
            && self.deleted_static_files == 0
    }
}
