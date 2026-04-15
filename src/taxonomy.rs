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

pub fn should_replace_display_name(current: &str, candidate: &str) -> bool {
    let current_score = display_name_score(current);
    let candidate_score = display_name_score(candidate);
    candidate_score > current_score
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

fn display_name_score(name: &str) -> i32 {
    let has_upper = name.chars().any(|ch| ch.is_uppercase());
    let has_lower = name.chars().any(|ch| ch.is_lowercase());
    let starts_upper = name.chars().next().map(|ch| ch.is_uppercase()).unwrap_or(false);

    let mut score = 0;
    if has_upper && has_lower {
        score += 20;
    }
    if starts_upper {
        score += 10;
    }
    if has_upper {
        score += 5;
    }
    if has_lower {
        score += 1;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::{should_replace_display_name, slugify};

    #[test]
    fn slugifies_special_characters() {
        assert_eq!(slugify("C++"), "c-plus-plus");
        assert_eq!(slugify("Windows 程序"), "windows-程序");
    }

    #[test]
    fn prefers_better_display_names() {
        assert!(should_replace_display_name("rust", "Rust"));
        assert!(should_replace_display_name("postgresql", "PostgreSQL"));
        assert!(!should_replace_display_name("Rust", "rust"));
    }
}
