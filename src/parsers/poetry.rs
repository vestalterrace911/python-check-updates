use anyhow::Result;
use std::path::Path;

use crate::parsers::{Dependency, DependencyParser};

pub struct PoetryParser;

impl DependencyParser for PoetryParser {
    fn parse(&self, path: &Path) -> Result<Vec<Dependency>> {
        let content = std::fs::read_to_string(path)?;
        let toml: toml::Value = toml::from_str(&content)?;

        let Some(poetry) = toml.get("tool").and_then(|t| t.get("poetry")) else {
            return Ok(vec![]);
        };

        let mut deps = Vec::new();

        // [tool.poetry.dependencies] and [tool.poetry.dev-dependencies] (Poetry 1.x legacy)
        for key in ["dependencies", "dev-dependencies"] {
            if let Some(t) = poetry.get(key).and_then(|d| d.as_table()) {
                deps.extend(table_deps(t));
            }
        }

        // [tool.poetry.group.*.dependencies] (Poetry 1.2+)
        if let Some(groups) = poetry.get("group").and_then(|g| g.as_table()) {
            for group in groups.values() {
                if let Some(t) = group.get("dependencies").and_then(|d| d.as_table()) {
                    deps.extend(table_deps(t));
                }
            }
        }

        Ok(deps)
    }
}

fn table_deps(
    table: &toml::map::Map<String, toml::Value>,
) -> impl Iterator<Item = Dependency> + '_ {
    table
        .iter()
        .filter(|(name, _)| name.as_str() != "python")
        .map(|(name, version)| Dependency {
            name: name.clone(),
            constraint: extract_constraint(version),
        })
}

fn extract_constraint(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string(),
        _ => "*".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_poetry_fixture() {
        let path = Path::new("tests/fixtures/poetry_pyproject.toml");
        let deps = PoetryParser.parse(path).unwrap();
        assert!(!deps.is_empty());
        assert!(deps.iter().any(|d| d.name == "fastapi"));
        assert!(deps.iter().any(|d| d.name == "pytest"));
        assert!(!deps.iter().any(|d| d.name == "python")); // python is skipped
    }

    #[test]
    fn test_extract_constraint_string() {
        let v = toml::Value::String("^1.2.3".to_string());
        assert_eq!(extract_constraint(&v), "^1.2.3");
    }

    #[test]
    fn test_extract_constraint_table() {
        let mut map = toml::map::Map::new();
        map.insert(
            "version".to_string(),
            toml::Value::String(">=2.0".to_string()),
        );
        map.insert("optional".to_string(), toml::Value::Boolean(true));
        assert_eq!(extract_constraint(&toml::Value::Table(map)), ">=2.0");
    }

    #[test]
    fn test_extract_constraint_table_no_version() {
        let map = toml::map::Map::new();
        assert_eq!(extract_constraint(&toml::Value::Table(map)), "*");
    }

    #[test]
    fn test_extract_constraint_other() {
        assert_eq!(extract_constraint(&toml::Value::Boolean(true)), "*");
    }

    #[test]
    fn test_parse_no_poetry_section_returns_empty() {
        // PoetryParser on a non-Poetry file hits the let-else early return.
        let path = Path::new("tests/fixtures/uv_pyproject.toml");
        let deps = PoetryParser.parse(path).unwrap();
        assert!(deps.is_empty());
    }
}
