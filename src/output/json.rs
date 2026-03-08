use anyhow::Result;
use serde::Serialize;

use crate::version::compare::Update;

#[derive(Serialize)]
struct JsonUpdate<'a> {
    name: &'a str,
    current: &'a str,
    latest: &'a str,
}

pub fn print_json(updates: &[Update]) -> Result<()> {
    println!("{}", to_json_string(updates)?);
    Ok(())
}

fn to_json_string(updates: &[Update]) -> Result<String> {
    let items: Vec<JsonUpdate> = updates
        .iter()
        .map(|u| JsonUpdate {
            name: &u.name,
            current: &u.current,
            latest: &u.latest,
        })
        .collect();
    Ok(serde_json::to_string_pretty(&items)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::compare::{BumpKind, Update};

    fn make_update(name: &str, current: &str, latest: &str) -> Update {
        Update {
            name: name.to_string(),
            current: current.to_string(),
            latest: latest.to_string(),
            updated_constraint: format!(">={}", latest),
            bump_kind: BumpKind::Minor,
        }
    }

    #[test]
    fn test_to_json_empty() {
        let json = to_json_string(&[]).unwrap();
        assert_eq!(json.trim(), "[]");
    }

    #[test]
    fn test_to_json_fields() {
        let updates = vec![make_update("fastapi", ">=0.109.0", "0.135.1")];
        let json = to_json_string(&updates).unwrap();
        assert!(json.contains("\"fastapi\""));
        assert!(json.contains("\">=0.109.0\""));
        assert!(json.contains("\"0.135.1\""));
    }

    #[test]
    fn test_to_json_multiple() {
        let updates = vec![
            make_update("fastapi", ">=0.109.0", "0.135.1"),
            make_update("pydantic", ">=1.0.0", "2.0.0"),
        ];
        let json = to_json_string(&updates).unwrap();
        assert!(json.contains("\"fastapi\""));
        assert!(json.contains("\"pydantic\""));
    }

    #[test]
    fn test_print_json_does_not_panic() {
        let updates = vec![make_update("ruff", "==0.1.6", "0.5.0")];
        print_json(&updates).unwrap();
        print_json(&[]).unwrap();
    }
}
