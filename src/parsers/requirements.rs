use anyhow::Result;
use std::path::Path;

use crate::parsers::{Dependency, DependencyParser, parse_pep508};

pub struct RequirementsTxtParser;

impl DependencyParser for RequirementsTxtParser {
    fn parse(&self, path: &Path) -> Result<Vec<Dependency>> {
        let content = std::fs::read_to_string(path)?;
        let mut deps = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip blank lines, comments, and pip directives (-r, -c, -e, --index-url, etc.)
            if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
                continue;
            }

            // Strip inline comment
            let line = line.split('#').next().unwrap_or(line).trim();

            if let Some(dep) = parse_pep508(line) {
                deps.push(dep);
            }
        }

        Ok(deps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirements_fixture() {
        let path = Path::new("tests/fixtures/requirements.txt");
        let parser = RequirementsTxtParser;
        let deps = parser.parse(path).unwrap();
        assert!(deps.iter().any(|d| d.name == "fastapi"));
        assert!(deps.iter().any(|d| d.name == "pydantic"));
        assert!(deps.iter().any(|d| d.name == "pytest"));
        assert!(deps.iter().any(|d| d.name == "ruff"));
    }

    #[test]
    fn test_skips_comments_and_directives() {
        let path = Path::new("tests/fixtures/requirements.txt");
        let parser = RequirementsTxtParser;
        let deps = parser.parse(path).unwrap();
        // These should not appear as deps
        assert!(!deps.iter().any(|d| d.name.starts_with('-')));
        assert!(!deps.iter().any(|d| d.name.contains("://")));
    }

    #[test]
    fn test_inline_comment_stripped() {
        let path = Path::new("tests/fixtures/requirements.txt");
        let parser = RequirementsTxtParser;
        let deps = parser.parse(path).unwrap();
        // Constraints should not contain comment text
        for dep in &deps {
            assert!(!dep.constraint.contains('#'));
        }
    }
}
