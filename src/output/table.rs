use owo_colors::OwoColorize;

use crate::version::compare::Update;
use crate::version::constraint::{extract_base_version, get_prefix};

pub fn print_table(updates: &[Update], summary: bool) {
    if updates.is_empty() {
        println!("All dependencies are up to date.");
        return;
    }

    let name_w = updates
        .iter()
        .map(|u| u.name.len())
        .max()
        .unwrap_or(8)
        .max(8);
    let curr_w = updates
        .iter()
        .map(|u| u.current.len())
        .max()
        .unwrap_or(8)
        .max(8);

    println!();
    for u in updates {
        let name_padded = format!("{:<width$}", u.name, width = name_w);
        let curr_padded = format!("{:>width$}", u.current, width = curr_w);
        let colored_new = color_updated_constraint(&u.current, &u.latest, &u.updated_constraint);
        println!(
            "{}  {}  {}  {}",
            name_padded.bold(),
            curr_padded.dimmed(),
            "→".cyan(),
            colored_new,
        );
    }
    println!();

    if summary {
        let n = updates.len();
        if n == 1 {
            println!("1 package can be updated.");
        } else {
            println!("{} packages can be updated.", n);
        }
    }
}

/// Color the updated constraint based on which version component changed:
/// - Major (X): new version is red
/// - Minor (Y): X. is plain, Y.Z... is blue
/// - Patch (Z): X.Y. is plain, Z... is green
///
/// For compound constraints (`>=new,<upper`) the upper bound is appended dimmed.
fn color_updated_constraint(current: &str, latest: &str, updated_constraint: &str) -> String {
    let old_base = match extract_base_version(current) {
        Some(v) => v,
        None => return updated_constraint.to_string(),
    };

    let old_parts: Vec<&str> = old_base.split('.').collect();
    let new_parts: Vec<&str> = latest.split('.').collect();

    // Index of the first differing component
    let diff_idx = old_parts
        .iter()
        .zip(new_parts.iter())
        .position(|(a, b)| a != b)
        .unwrap_or(new_parts.len());

    // Build the colored lower-bound portion (prefix + colored version)
    let lower_part = if updated_constraint.contains(',') {
        // Compound: first component is ">=new_version"
        updated_constraint
            .split_once(',')
            .map_or(updated_constraint, |(before, _)| before)
    } else {
        updated_constraint
    };
    let prefix = get_prefix(lower_part);

    let colored_version = match diff_idx {
        0 => latest.red().to_string(),
        1 => {
            let plain = format!("{}.", new_parts[0]);
            let colored = new_parts[1..].join(".").blue().to_string();
            format!("{}{}", plain, colored)
        }
        2 => {
            let plain = format!("{}.{}.", new_parts[0], new_parts[1]);
            let colored = new_parts[2..].join(".").green().to_string();
            format!("{}{}", plain, colored)
        }
        _ => latest.to_string(),
    };

    // Append the upper bound dimmed if present
    let upper_part = if let Some(comma) = updated_constraint.find(',') {
        updated_constraint[comma..].to_string().dimmed().to_string()
    } else {
        String::new()
    };

    format!("{}{}{}", prefix, colored_version, upper_part)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::compare::{BumpKind, Update};

    fn make_update(
        name: &str,
        current: &str,
        latest: &str,
        updated: &str,
        bump: BumpKind,
    ) -> Update {
        Update {
            name: name.to_string(),
            current: current.to_string(),
            latest: latest.to_string(),
            updated_constraint: updated.to_string(),
            bump_kind: bump,
        }
    }

    #[test]
    fn test_print_table_empty() {
        print_table(&[], false);
        print_table(&[], true);
    }

    #[test]
    fn test_print_table_with_updates() {
        let updates = vec![
            make_update(
                "fastapi",
                ">=0.109.0",
                "0.135.1",
                ">=0.135.1",
                BumpKind::Minor,
            ),
            make_update("pydantic", ">=1.0.0", "2.0.0", ">=2.0.0", BumpKind::Major),
        ];
        print_table(&updates, true);
        print_table(&updates, false);
    }

    #[test]
    fn test_print_table_single_package_summary() {
        // Exercises the `n == 1` branch: "1 package can be updated."
        let updates = vec![make_update(
            "fastapi",
            ">=0.109.0",
            "0.135.1",
            ">=0.135.1",
            BumpKind::Minor,
        )];
        print_table(&updates, true);
    }

    #[test]
    fn test_color_updated_constraint_major() {
        let result = color_updated_constraint(">=1.0.0", "2.0.0", ">=2.0.0");
        // Major bump: entire version is colored red; "2.0.0" is still a substring
        assert!(result.contains("2.0.0"));
        assert!(result.contains(">="));
    }

    #[test]
    fn test_color_updated_constraint_minor() {
        let result = color_updated_constraint(">=0.109.0", "0.110.0", ">=0.110.0");
        // Minor bump: prefix "0." is plain, "110.0" colored blue
        assert!(result.contains("0."));
        assert!(result.contains("110"));
    }

    #[test]
    fn test_color_updated_constraint_patch() {
        let result = color_updated_constraint(">=1.0.0", "1.0.1", ">=1.0.1");
        // Patch bump: "1.0." is plain, "1" (the new patch) colored green
        assert!(result.contains("1.0."));
    }

    #[test]
    fn test_color_updated_constraint_compound() {
        let result = color_updated_constraint(">=0.7.3,<0.8.0", "1.0.0", ">=1.0.0,<2.0.0");
        assert!(result.contains("1.0.0"));
        assert!(result.contains("2.0.0")); // upper bound present (dimmed but still a substring)
    }

    #[test]
    fn test_color_updated_constraint_no_base_version() {
        // Bare wildcard - returns the constraint as-is
        let result = color_updated_constraint("*", "1.0.0", "*");
        assert_eq!(result, "*");
    }

    #[test]
    fn test_color_updated_constraint_four_component_version() {
        // diff_idx falls through to the `_` arm: first 3 components are equal,
        // difference is in the 4th component → rendered uncolored.
        let result = color_updated_constraint(">=1.0.0.0", "1.0.0.1", ">=1.0.0.1");
        assert!(result.contains("1.0.0.1"));
        assert!(result.contains(">="));
    }
}
