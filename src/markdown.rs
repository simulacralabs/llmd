//! Markdown utilities: section extraction, heading listing, token estimation.
//!
//! Uses a line-oriented approach — fast and simple for the structured markdown
//! that .llmd/ files are expected to contain.

/// Extracts the content of the first H2 or H3 section whose heading text
/// contains `section` as a case-insensitive substring. Returns the raw
/// markdown text of that section, stopping at the next heading of equal or
/// higher level.
///
/// Returns `None` if no matching section is found.
pub fn extract_section(source: &str, section: &str) -> Option<String> {
    let needle = section.to_lowercase();
    let lines: Vec<&str> = source.lines().collect();
    let mut start_line: Option<usize> = None;
    let mut target_depth: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let depth = heading_depth(line);
        if depth == 0 {
            continue;
        }
        let heading_text = line.trim_start_matches('#').trim().to_lowercase();
        if start_line.is_none() && heading_text.contains(&needle) {
            start_line = Some(i);
            target_depth = depth;
        } else if let Some(start) = start_line {
            if depth <= target_depth && i > start {
                return Some(lines[start..i].join("\n"));
            }
        }
    }

    start_line.map(|start| lines[start..].join("\n"))
}

/// Returns a list of all headings in `source` as `(depth, text)` pairs.
pub fn list_headings(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .filter_map(|line| {
            let depth = heading_depth(line);
            if depth > 0 {
                Some((depth, line.trim_start_matches('#').trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// Estimates the number of tokens in `text` using the heuristic of 1 token per
/// 4 characters (a conservative approximation for English prose and code).
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

/// Returns lines `start..=end` (1-indexed) from `source`.
/// Clamps to the actual line range if out of bounds.
pub fn window(source: &str, start: usize, end: usize) -> String {
    source
        .lines()
        .skip(start.saturating_sub(1))
        .take(end.saturating_sub(start.saturating_sub(1)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Returns the heading depth of a line (1 for `#`, 2 for `##`, etc.),
/// or 0 if the line is not a heading.
fn heading_depth(line: &str) -> usize {
    let trimmed = line.trim_start_matches('#');
    let depth = line.len() - trimmed.len();
    if depth > 0 && trimmed.starts_with(' ') {
        depth
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_depth() {
        assert_eq!(heading_depth("# Foo"), 1);
        assert_eq!(heading_depth("## Bar"), 2);
        assert_eq!(heading_depth("### Baz"), 3);
        assert_eq!(heading_depth("not a heading"), 0);
        assert_eq!(heading_depth("##no space"), 0);
    }

    #[test]
    fn test_extract_section() {
        let md = "# Top\n\n## Alpha\n\nalpha content\n\n## Beta\n\nbeta content\n";
        let result = extract_section(md, "Alpha").unwrap();
        assert!(result.contains("alpha content"));
        assert!(!result.contains("beta content"));
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens("1234"), 1);
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_list_headings() {
        let md = "# Top\n\n## Sub\n\nsome text\n\n### Deep\n";
        let headings = list_headings(md);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0], (1, "Top".to_string()));
        assert_eq!(headings[1], (2, "Sub".to_string()));
        assert_eq!(headings[2], (3, "Deep".to_string()));
    }

    #[test]
    fn test_window() {
        let text = "a\nb\nc\nd\ne";
        assert_eq!(window(text, 2, 4), "b\nc\nd");
    }
}
