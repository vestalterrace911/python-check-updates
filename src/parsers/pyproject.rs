use anyhow::Result;
use std::path::Path;

use crate::parsers::{Dependency, DependencyParser, parse_pep508};

pub struct PyProjectParser;

impl DependencyParser for PyProjectParser {
    fn parse(&self, path: &Path) -> Result<Vec<Dependency>> {
        let content = std::fs::read_to_string(path)?;
        let toml: toml::Value = toml::from_str(&content)?;

        let mut deps = Vec::new();
        let project = toml.get("project");

        // [project.dependencies]
        if let Some(arr) = project
            .and_then(|p| p.get("dependencies"))
            .and_then(|d| d.as_array())
        {
            deps.extend(pep508_from_array(arr));
        }

        // [project.optional-dependencies.*]
        if let Some(groups) = project
            .and_then(|p| p.get("optional-dependencies"))
            .and_then(|d| d.as_table())
        {
            deps.extend(pep508_from_groups(groups));
        }

        // [dependency-groups.*] (PEP 735, used by uv)
        // String entries are parsed; inline tables (e.g. {include-group = "..."}) are silently skipped.
        if let Some(groups) = toml.get("dependency-groups").and_then(|d| d.as_table()) {
            deps.extend(pep508_from_groups(groups));
        }

        Ok(deps)
    }
}

fn pep508_from_array(arr: &[toml::Value]) -> impl Iterator<Item = Dependency> + '_ {
    arr.iter().filter_map(|v| v.as_str().and_then(parse_pep508))
}

fn pep508_from_groups(table: &toml::map::Map<String, toml::Value>) -> Vec<Dependency> {
    table
        .values()
        .filter_map(|v| v.as_array())
        .flat_map(|arr| arr.iter().filter_map(|v| v.as_str().and_then(parse_pep508)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pyproject_fixture() {
        let path = Path::new("tests/fixtures/uv_pyproject.toml");
        let deps = PyProjectParser.parse(path).unwrap();
        assert!(!deps.is_empty());
        assert!(deps.iter().any(|d| d.name == "fastapi"));
        assert!(deps.iter().any(|d| d.name == "pytest")); // from optional-dependencies
    }

    #[test]
    fn test_constraint_preserved() {
        let path = Path::new("tests/fixtures/uv_pyproject.toml");
        let deps = PyProjectParser.parse(path).unwrap();
        let fastapi = deps.iter().find(|d| d.name == "fastapi").unwrap();
        assert_eq!(fastapi.constraint, ">=0.109.0");
    }

    #[test]
    fn test_parse_dependency_groups_fixture() {
        let path = Path::new("tests/fixtures/uv_dependency_groups_pyproject.toml");
        let deps = PyProjectParser.parse(path).unwrap();
        assert!(deps.iter().any(|d| d.name == "loguru"));
        assert!(deps.iter().any(|d| d.name == "ruff"));
        assert!(deps.iter().any(|d| d.name == "mypy"));
    }

    #[test]
    fn test_compound_constraint_parsed() {
        let path = Path::new("tests/fixtures/uv_dependency_groups_pyproject.toml");
        let deps = PyProjectParser.parse(path).unwrap();
        let loguru = deps.iter().find(|d| d.name == "loguru").unwrap();
        assert_eq!(loguru.constraint, ">=0.7.3,<0.8.0");
    }
}
