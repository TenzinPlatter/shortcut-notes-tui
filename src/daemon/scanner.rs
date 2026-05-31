use std::path::{Path, PathBuf};

use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTodo {
    pub text: String,
    pub completed: bool,
    pub date: Option<NaiveDate>,
    pub file: PathBuf,
    pub line: u32,
    pub fingerprint: u64,
}

pub fn parse_note(content: &str, file: &Path) -> Vec<ParsedTodo> {
    let mut out = Vec::new();
    let date_re_simple = |s: &str| -> Option<(NaiveDate, std::ops::Range<usize>)> {
        // Find first @DD-MM-YYYY token (literally @ then 2-2-4 digits).
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'@' && i + 11 <= bytes.len() {
                let candidate = &s[i + 1..i + 11];
                if candidate.as_bytes().iter().enumerate().all(|(idx, b)| {
                    matches!(
                        (idx, b),
                        (0..=1, b'0'..=b'9')
                            | (2, b'-')
                            | (3..=4, b'0'..=b'9')
                            | (5, b'-')
                            | (6..=9, b'0'..=b'9')
                    )
                }) && let Ok(d) = NaiveDate::parse_from_str(candidate, "%d-%m-%Y")
                {
                    return Some((d, i..i + 11));
                }
            }
            i += 1;
        }
        None
    };

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = (idx as u32) + 1;
        let trimmed = raw_line.trim_start();
        let (completed, rest) = if let Some(r) = trimmed.strip_prefix("- [ ] ") {
            (false, r)
        } else if let Some(r) = trimmed.strip_prefix("- [x] ") {
            (true, r)
        } else if let Some(r) = trimmed.strip_prefix("- [X] ") {
            (true, r)
        } else {
            continue;
        };

        let (date, text) = match date_re_simple(rest) {
            Some((d, range)) => {
                let mut s = String::with_capacity(rest.len() - 11);
                s.push_str(&rest[..range.start]);
                s.push_str(&rest[range.end..]);
                let cleaned = s
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                (Some(d), cleaned)
            }
            None => {
                let cleaned = rest
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                (None, cleaned)
            }
        };

        let normalized = text.to_lowercase();
        let fp = fingerprint(file, &normalized);

        out.push(ParsedTodo {
            text,
            completed,
            date,
            file: file.to_path_buf(),
            line: line_no,
            fingerprint: fp,
        });
    }

    out
}

/// FNV-1a 64-bit hash over the file path + normalized text.
/// Hardcoded so fingerprint values remain stable across Rust toolchain upgrades
/// (DefaultHasher does not guarantee this).
pub fn fingerprint(file: &Path, normalized_text: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;
    let mut hash = FNV_OFFSET;
    for b in file.to_string_lossy().as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    // Separator byte so e.g. ("ab.md", "c") doesn't collide with ("ab.md\0c", "").
    hash ^= 0xff;
    hash = hash.wrapping_mul(FNV_PRIME);
    for b in normalized_text.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f() -> PathBuf {
        PathBuf::from("note.md")
    }

    #[test]
    fn parses_single_open_checkbox() {
        let t = parse_note("- [ ] write the thing", &f());
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].text, "write the thing");
        assert!(!t[0].completed);
        assert_eq!(t[0].date, None);
        assert_eq!(t[0].line, 1);
    }

    #[test]
    fn parses_completed_checkbox_case_insensitive() {
        let t = parse_note("- [x] done\n- [X] also done", &f());
        assert_eq!(t.len(), 2);
        assert!(t[0].completed);
        assert!(t[1].completed);
    }

    #[test]
    fn extracts_dd_mm_yyyy_date_and_strips_from_text() {
        let t = parse_note("- [ ] follow up @10-06-2026 please", &f());
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].date, Some(NaiveDate::from_ymd_opt(2026, 6, 10).unwrap()));
        assert_eq!(t[0].text, "follow up please");
    }

    #[test]
    fn invalid_date_retained_in_text_no_date_field() {
        let t = parse_note("- [ ] todo @32-01-2026", &f());
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].date, None);
        assert_eq!(t[0].text, "todo @32-01-2026");
    }

    #[test]
    fn ignores_indentation_in_recognition() {
        let t = parse_note("    - [ ] indented", &f());
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].text, "indented");
    }

    #[test]
    fn multiple_todos_in_one_file_with_line_numbers() {
        let content = "# Header\n- [ ] one\nsome prose\n- [x] two\n";
        let t = parse_note(content, &f());
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].line, 2);
        assert_eq!(t[1].line, 4);
    }

    #[test]
    fn no_checkboxes_returns_empty() {
        let t = parse_note("# Title\n\njust prose.", &f());
        assert!(t.is_empty());
    }

    #[test]
    fn fingerprint_stable_across_whitespace_in_text() {
        let a = parse_note("- [ ] hello world", &f());
        let b = parse_note("    - [ ]   hello world   ", &f());
        assert_eq!(a[0].fingerprint, b[0].fingerprint);
    }

    #[test]
    fn fingerprint_changes_after_text_edit() {
        let a = parse_note("- [ ] hello", &f());
        let b = parse_note("- [ ] hello world", &f());
        assert_ne!(a[0].fingerprint, b[0].fingerprint);
    }

    #[test]
    fn fingerprint_stable_across_line_moves() {
        // Same text on different lines → same fingerprint
        let a = parse_note("- [ ] x", &f());
        let b = parse_note("# header\n\n- [ ] x", &f());
        assert_eq!(a[0].fingerprint, b[0].fingerprint);
        assert_ne!(a[0].line, b[0].line);
    }
}
