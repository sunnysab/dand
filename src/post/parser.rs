use anyhow::{anyhow, bail, Result};

use super::RawFrontMatter;

pub fn split(full_content: &str) -> Result<(RawFrontMatter, &str)> {
    let raw_content = full_content.trim_start();

    fn find_header_end(raw_content: &str, separator: &str) -> Result<usize> {
        raw_content[4..]
            .find(separator)
            .map(|x| x + 4)
            .ok_or_else(|| anyhow!("the end separator of the header is not found."))
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
        bail!("no front header detected.");
    };
    let header = header.map_err(|e| anyhow!("Could not parse the front matter, {e}"))?;
    Ok((header, &full_content[end + 4..]))
}
