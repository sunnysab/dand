use std::rc::Rc;

use anyhow::{anyhow, bail, Context, Result};

use super::tag::SharedTag;
use super::{Post, PostFlag, PostMetadata, PostReference, RawFrontMatter};

fn split(full_content: &str) -> Result<(RawFrontMatter, &str)> {
    let raw_content = full_content.trim_start();

    fn find_header_end(raw_content: &str, separator: &str) -> Result<usize> {
        4 + raw_content[4..]
            .find(separator)
            .ok_or_else(|| anyhow!("Could not find the end of the header"))
    }

    let (header, end): (Result<RawFrontMatter>, usize) = if raw_content.starts_with("---\n") {
        let end = find_header_end(full_content, "---")?;
        let header =
            serde_yaml::from_str::<RawFrontMatter>(&raw_content[4..end]).map_err(Into::into);

        (header, end)
    } else if raw_content.starts_with("+++\n") {
        let end = find_header_end(full_content, "+++")?;
        let header = toml::from_str::<RawFrontMatter>(&raw_content[4..end]).map_err(Into::into);

        (header, end)
    } else {
        bail!("Could not find the start of the header");
    };
    let header = header.with_context("Could not parse the front matter")?;
    Ok((header, &full_content[end + 9..]))
}
