pub mod poetry;
pub mod pyproject;
pub mod requirements;

use anyhow::Result;
use std::path::Path;

use crate::parsers::poetry::PoetryParser;
use crate::parsers::pyproject::PyProjectParser;
use crate::parsers::requirements::RequirementsTxtParser;

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub constraint: String,
}

pub trait DependencyParser {
    fn parse(&self, path: &Path) -> Result<Vec<Dependency>>;
}

/// Detect which parser to use based on file name / contents.
pub fn detect_parser(path: &Path) -> Result<Box<dyn DependencyParser>> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // Any .txt file (requirements.txt, requirements-dev.txt, etc.)
    if name.ends_with(".txt") {
        return Ok(Box::new(RequirementsTxtParser));
    }

    // TOML files: peek at contents to distinguish Poetry from PEP 621
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Could not read {}: {}", path.display(), e))?;
    let toml: toml::Value = toml::from_str(&content)?;

    if toml.get("tool").and_then(|t| t.get("poetry")).is_some() {
        Ok(Box::new(PoetryParser))
    } else {
        Ok(Box::new(PyProjectParser))
    }
}

/// Parse a PEP 508 dependency string. Returns `None` for URL/VCS/local-path deps
/// and for empty or whitespace-only input.
pub(crate) fn parse_pep508(s: &str) -> Option<Dependency> {
    // Strip environment markers (everything after ';')
    let s = s.split_once(';').map_or(s, |(before, _)| before).trim();

    // Skip URL / VCS / local-path deps
    if s.contains("://") || s.starts_with('.') || s.starts_with('/') || s.contains(" @ ") {
        return None;
    }

    // Find end of package name (before extras, version specifiers, or whitespace)
    let name_end = s
        .find(['[', '>', '<', '=', '~', '!', '^', ' ', '\t'])
        .unwrap_or(s.len());

    let name = s[..name_end].trim().to_string();
    if name.is_empty() {
        return None;
    }

    // Skip over extras block [...]
    let rest = s[name_end..].trim();
    let rest = if rest.starts_with('[') {
        if let Some(end) = rest.find(']') {
            rest[end + 1..].trim()
        } else {
            rest
        }
    } else {
        rest
    };

    let constraint = rest.trim().to_string();
    Some(Dependency { name, constraint })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pep508_basic() {
        let d = parse_pep508("fastapi>=0.109.0").unwrap();
        assert_eq!(d.name, "fastapi");
        assert_eq!(d.constraint, ">=0.109.0");
    }

    #[test]
    fn test_parse_pep508_extras() {
        let d = parse_pep508("pydantic[email]>=2.0").unwrap();
        assert_eq!(d.name, "pydantic");
        assert_eq!(d.constraint, ">=2.0");
    }

    #[test]
    fn test_parse_pep508_multiple_extras() {
        let d = parse_pep508("pydantic[email,dotenv]>=2.0").unwrap();
        assert_eq!(d.name, "pydantic");
        assert_eq!(d.constraint, ">=2.0");
    }

    #[test]
    fn test_parse_pep508_env_marker() {
        let d = parse_pep508("requests>=2.28; python_version >= '3.8'").unwrap();
        assert_eq!(d.name, "requests");
        assert_eq!(d.constraint, ">=2.28");
    }

    #[test]
    fn test_parse_pep508_tilde_eq() {
        let d = parse_pep508("pytest~=7.3.0").unwrap();
        assert_eq!(d.name, "pytest");
        assert_eq!(d.constraint, "~=7.3.0");
    }

    #[test]
    fn test_parse_pep508_exact() {
        let d = parse_pep508("ruff==0.1.6").unwrap();
        assert_eq!(d.name, "ruff");
        assert_eq!(d.constraint, "==0.1.6");
    }

    #[test]
    fn test_parse_pep508_compound() {
        let d = parse_pep508("loguru>=0.7.3,<0.8.0").unwrap();
        assert_eq!(d.name, "loguru");
        assert_eq!(d.constraint, ">=0.7.3,<0.8.0");
    }

    #[test]
    fn test_parse_pep508_bare_name() {
        let d = parse_pep508("requests").unwrap();
        assert_eq!(d.name, "requests");
        assert_eq!(d.constraint, "");
    }

    #[test]
    fn test_parse_pep508_url_skipped() {
        assert!(parse_pep508("git+https://github.com/example/pkg.git").is_none());
    }

    #[test]
    fn test_parse_pep508_at_notation_skipped() {
        assert!(parse_pep508("pkg @ https://example.com/pkg.tar.gz").is_none());
    }

    #[test]
    fn test_parse_pep508_local_relative_skipped() {
        assert!(parse_pep508("./local-package").is_none());
    }

    #[test]
    fn test_parse_pep508_local_absolute_skipped() {
        assert!(parse_pep508("/usr/local/lib/pkg").is_none());
    }

    #[test]
    fn test_parse_pep508_empty_skipped() {
        assert!(parse_pep508("").is_none());
        assert!(parse_pep508("   ").is_none());
    }

    #[test]
    fn test_detect_parser_requirements_txt() {
        let path = Path::new("tests/fixtures/requirements.txt");
        let parser = detect_parser(path).unwrap();
        let deps = parser.parse(path).unwrap();
        assert!(!deps.is_empty());
        assert!(deps.iter().any(|d| d.name == "fastapi"));
    }

    #[test]
    fn test_detect_parser_poetry() {
        let path = Path::new("tests/fixtures/poetry_pyproject.toml");
        let parser = detect_parser(path).unwrap();
        let deps = parser.parse(path).unwrap();
        assert!(deps.iter().any(|d| d.name == "fastapi"));
        assert!(!deps.iter().any(|d| d.name == "python"));
    }

    #[test]
    fn test_detect_parser_pep621() {
        let path = Path::new("tests/fixtures/uv_pyproject.toml");
        let parser = detect_parser(path).unwrap();
        let deps = parser.parse(path).unwrap();
        assert!(deps.iter().any(|d| d.name == "fastapi"));
    }

    #[test]
    fn test_detect_parser_nonexistent() {
        assert!(detect_parser(Path::new("nonexistent.toml")).is_err());
    }
}
