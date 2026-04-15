use std::collections::{BTreeMap, HashMap, HashSet};

pub type SlugMap = HashMap<String, String>;

pub fn build_slug_map<'a>(names: impl IntoIterator<Item = &'a str>) -> SlugMap {
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in names {
        grouped
            .entry(slugify(name))
            .or_default()
            .push(name.to_string());
    }

    let mut used = HashSet::new();
    let mut slugs = HashMap::new();
    for (base, mut names) in grouped {
        names.sort_by_key(|name| {
            (
                name.as_str() != base.as_str(),
                name.to_lowercase(),
                name.clone(),
            )
        });

        if names.len() == 1 {
            let slug = reserve_slug(base, &mut used);
            slugs.insert(names.remove(0), slug);
            continue;
        }

        for (index, name) in names.into_iter().enumerate() {
            let preferred = if index == 0 {
                base.clone()
            } else {
                format!("{base}-{}", index + 1)
            };
            let slug = reserve_slug(preferred, &mut used);
            slugs.insert(name, slug);
        }
    }

    slugs
}

pub fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut prev_separator = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_separator = false;
        } else if ch.is_alphanumeric() {
            slug.push(ch);
            prev_separator = false;
        } else {
            match ch {
                '+' => push_token(&mut slug, &mut prev_separator, "plus"),
                '#' => push_token(&mut slug, &mut prev_separator, "sharp"),
                '&' => push_token(&mut slug, &mut prev_separator, "and"),
                '-' | '_' | '/' | '.' => push_separator(&mut slug, &mut prev_separator),
                _ if ch.is_whitespace() => push_separator(&mut slug, &mut prev_separator),
                _ => push_separator(&mut slug, &mut prev_separator),
            }
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        String::from("item")
    } else {
        trimmed
    }
}

pub fn slug_for<'a>(name: &'a str, slugs: &'a SlugMap) -> &'a str {
    slugs.get(name).map(String::as_str).unwrap_or(name)
}

fn reserve_slug(preferred: String, used: &mut HashSet<String>) -> String {
    if used.insert(preferred.clone()) {
        return preferred;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{preferred}-{suffix}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

fn push_separator(slug: &mut String, prev_separator: &mut bool) {
    if slug.is_empty() || *prev_separator {
        return;
    }

    slug.push('-');
    *prev_separator = true;
}

fn push_token(slug: &mut String, prev_separator: &mut bool, token: &str) {
    if !slug.is_empty() && !*prev_separator {
        slug.push('-');
    }
    slug.push_str(token);
    *prev_separator = false;
}

#[cfg(test)]
mod tests {
    use super::{build_slug_map, slugify};

    #[test]
    fn slugifies_special_characters() {
        assert_eq!(slugify("C++"), "c-plus-plus");
        assert_eq!(slugify("Windows 程序"), "windows-程序");
    }

    #[test]
    fn assigns_stable_suffixes_for_collisions() {
        let map = build_slug_map(["Rust", "rust"]);

        assert_eq!(map.get("rust").unwrap(), "rust");
        assert_eq!(map.get("Rust").unwrap(), "rust-2");
    }
}
