use anyhow::Result;
use std::path::Path;

use crate::version::compare::Update;

/// Rewrite `path` in-place, replacing each outdated constraint with its updated value.
/// Returns the number of constraints actually replaced.
pub fn apply_upgrades(path: &Path, updates: &[Update]) -> Result<usize> {
    if updates.is_empty() {
        return Ok(0);
    }

    let mut content = std::fs::read_to_string(path)?;
    let mut count = 0;

    for update in updates {
        let (new_content, replaced) = replace_constraint(
            &content,
            &update.name,
            &update.current,
            &update.updated_constraint,
        );
        if replaced {
            content = new_content;
            count += 1;
        }
    }

    std::fs::write(path, &content)?;
    Ok(count)
}

/// Find the first occurrence of `current` that appears within a short window after
/// `name` in `content`, and replace it with `updated`. Returns the new content and
/// whether a replacement was made.
///
/// The window approach handles both formats:
/// - PEP 508 strings:  `"fastapi>=0.109.0"`        → name then constraint immediately
/// - PEP 508 extras:   `"pydantic[email]>=2.0"`     → name, extras, then constraint
/// - Poetry table:     `fastapi = "^0.109.0"`       → name then ` = "` then constraint
/// - Poetry inline:    `{version = "^1.0", ...}`    → constraint inside inline table
fn replace_constraint(content: &str, name: &str, current: &str, updated: &str) -> (String, bool) {
    let mut search_from = 0;

    while let Some(rel_pos) = content[search_from..].find(name) {
        let name_start = search_from + rel_pos;
        let name_end = name_start + name.len();

        // Reject mid-word matches (e.g. "my-fastapi" should not match "fastapi")
        if name_start > 0 {
            let prev = content.as_bytes()[name_start - 1] as char;
            if prev.is_alphanumeric() || prev == '-' || prev == '_' || prev == '.' {
                search_from = name_start + 1;
                continue;
            }
        }

        // Look for `current` within a 60-char window after the name.
        // Large enough to cover `[some-extra-group] = "` prefixes.
        let window_end = (name_end + 60).min(content.len());
        let window = &content[name_end..window_end];

        if let Some(c_rel) = window.find(current) {
            let c_start = name_end + c_rel;
            let c_end = c_start + current.len();
            let new_content = format!("{}{}{}", &content[..c_start], updated, &content[c_end..]);
            return (new_content, true);
        }

        search_from = name_start + 1;
    }

    (content.to_string(), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_pep508() {
        let content = r#"dependencies = ["fastapi>=0.109.0", "pydantic>=1.10.0"]"#;
        let (new, replaced) = replace_constraint(content, "fastapi", ">=0.109.0", ">=0.135.1");
        assert!(replaced);
        assert!(new.contains("fastapi>=0.135.1"));
        assert!(new.contains("pydantic>=1.10.0")); // untouched
    }

    #[test]
    fn test_replace_pep508_with_extras() {
        let content = r#"dependencies = ["pydantic[email]>=2.0"]"#;
        let (new, replaced) = replace_constraint(content, "pydantic", ">=2.0", ">=2.12.5");
        assert!(replaced);
        assert!(new.contains("pydantic[email]>=2.12.5"));
    }

    #[test]
    fn test_replace_poetry() {
        let content = "fastapi = \"^0.109.0\"\npydantic = \"^1.10.0\"\n";
        let (new, replaced) = replace_constraint(content, "fastapi", "^0.109.0", "^0.135.1");
        assert!(replaced);
        assert!(new.contains("fastapi = \"^0.135.1\""));
        assert!(new.contains("pydantic = \"^1.10.0\"")); // untouched
    }

    #[test]
    fn test_replace_compound_constraint() {
        let content = r#"dependencies = ["loguru>=0.7.3,<0.8.0"]"#;
        let (new, replaced) = replace_constraint(content, "loguru", ">=0.7.3,<0.8.0", ">=0.8.0");
        assert!(replaced);
        assert!(new.contains("loguru>=0.8.0"));
    }

    #[test]
    fn test_apply_upgrades_empty_returns_zero() {
        // Empty updates slice must return Ok(0) without touching the file.
        let path = Path::new("tests/fixtures/uv_pyproject.toml");
        let count = apply_upgrades(path, &[]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_apply_upgrades_rewrites_file() {
        use crate::version::compare::{BumpKind, Update};

        let dir = std::env::temp_dir();
        let path = dir.join(format!("pycu_test_{}.txt", std::process::id()));
        std::fs::write(&path, "fastapi>=0.109.0\n").unwrap();

        let update = Update {
            name: "fastapi".to_string(),
            current: ">=0.109.0".to_string(),
            latest: "0.135.1".to_string(),
            updated_constraint: ">=0.135.1".to_string(),
            bump_kind: BumpKind::Minor,
        };

        let count = apply_upgrades(&path, &[update]).unwrap();
        assert_eq!(count, 1);
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains(">=0.135.1"));
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_replace_name_not_in_content() {
        // Name never appears → while loop never enters → hits (content.to_string(), false).
        let content = r#"dependencies = ["requests>=2.0"]"#;
        let (new, replaced) = replace_constraint(content, "fastapi", ">=0.109.0", ">=0.135.1");
        assert!(!replaced);
        assert_eq!(new, content);
    }

    #[test]
    fn test_no_mid_word_match() {
        let content = r#"dependencies = ["my-fastapi>=1.0", "fastapi>=0.109.0"]"#;
        let (new, replaced) = replace_constraint(content, "fastapi", ">=0.109.0", ">=0.135.1");
        assert!(replaced);
        // Only the standalone "fastapi" should be updated
        assert!(new.contains("my-fastapi>=1.0")); // untouched
        assert!(new.contains("\"fastapi>=0.135.1\""));
    }
}
