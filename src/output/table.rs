use owo_colors::OwoColorize;

use crate::cli::ColorScheme;
use crate::version::compare::Update;
use crate::version::constraint::{extract_base_version, get_prefix};

/// RGB triple for a single bump level.
struct Palette {
    major: (u8, u8, u8),
    minor: (u8, u8, u8),
    patch: (u8, u8, u8),
}

fn palette_for(scheme: &ColorScheme) -> Palette {
    match scheme {
        // GitHub-style SemVer severity
        ColorScheme::Default => Palette {
            major: (215, 58, 73), // #D73A49  red
            minor: (3, 102, 214), // #0366D6  blue
            patch: (40, 167, 69), // #28A745  green
        },
        // Okabe-Ito color-blind safe
        ColorScheme::OkabeIto => Palette {
            major: (230, 159, 0), // #E69F00  orange
            minor: (0, 114, 178), // #0072B2  blue
            patch: (0, 158, 115), // #009E73  teal
        },
        // Traffic-light
        ColorScheme::TrafficLight => Palette {
            major: (231, 76, 60),  // #E74C3C  red
            minor: (241, 196, 15), // #F1C40F  yellow
            patch: (46, 204, 113), // #2ECC71  green
        },
        // Monitoring / logging severity
        ColorScheme::Severity => Palette {
            major: (142, 68, 173),  // #8E44AD  purple
            minor: (52, 152, 219),  // #3498DB  blue
            patch: (149, 165, 166), // #95A5A6  gray
        },
        // High-contrast accessibility
        ColorScheme::HighContrast => Palette {
            major: (204, 121, 167), // #CC79A7  magenta
            minor: (0, 114, 178),   // #0072B2  blue
            patch: (240, 228, 66),  // #F0E442  yellow
        },
    }
}

pub fn print_table(updates: &[Update], summary: bool, color_scheme: &ColorScheme) {
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
        let colored_new =
            color_updated_constraint(&u.current, &u.latest, &u.updated_constraint, color_scheme);
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

/// Print a visual preview of every available color scheme.
/// Called by `--list-color-schemes` and on first run.
pub fn print_color_scheme_preview() {
    // (cli name, scheme, description, [major label, minor label, patch label])
    let schemes: &[(&str, ColorScheme, &str, [&str; 3])] = &[
        (
            "default",
            ColorScheme::Default,
            "SemVer severity - GitHub style",
            ["red #D73A49", "blue #0366D6", "green #28A745"],
        ),
        (
            "okabe-ito",
            ColorScheme::OkabeIto,
            "Color-blind safe - Okabe-Ito palette",
            ["orange #E69F00", "blue #0072B2", "teal #009E73"],
        ),
        (
            "traffic-light",
            ColorScheme::TrafficLight,
            "Traffic-light - common CI/dashboard model",
            ["red #E74C3C", "yellow #F1C40F", "green #2ECC71"],
        ),
        (
            "severity",
            ColorScheme::Severity,
            "Monitoring style - Datadog/Grafana inspired",
            ["purple #8E44AD", "blue #3498DB", "gray #95A5A6"],
        ),
        (
            "high-contrast",
            ColorScheme::HighContrast,
            "High-contrast accessibility - color-blind safe",
            ["magenta #CC79A7", "blue #0072B2", "yellow #F0E442"],
        ),
    ];

    // Sample rows: (current, latest, updated_constraint, bump label)
    let samples: &[(&str, &str, &str, &str)] = &[
        ("1.0.0", "2.0.0", "2.0.0", "major"),
        ("1.0.0", "1.1.0", "1.1.0", "minor"),
        ("1.0.0", "1.0.1", "1.0.1", "patch"),
    ];

    println!();
    println!(
        "{}",
        "Available color schemes  (use --set-color-scheme <SCHEME>):".bold()
    );

    for (name, scheme, description, color_labels) in schemes {
        println!();
        println!("  {}  -  {}", name.bold().underline(), description.dimmed());
        for (i, (current, latest, updated, bump)) in samples.iter().enumerate() {
            let colored = color_updated_constraint(current, latest, updated, scheme);
            let label_text = format!("{}  ({})", bump, color_labels[i]);
            let label = label_text.dimmed();
            println!(
                "    {}  {}  {}  {}  {}",
                "my-package".bold(),
                current.dimmed(),
                "→".cyan(),
                colored,
                label,
            );
        }
    }
    println!();
}

/// Color the updated constraint based on which version component changed.
///
/// The exact RGB color for each bump level comes from the active `ColorScheme`'s palette,
/// applied via true-color escape codes so the hex values are honored precisely.
///
/// For compound constraints (`>=new,<upper`) the upper bound is appended dimmed.
fn color_updated_constraint(
    current: &str,
    latest: &str,
    updated_constraint: &str,
    color_scheme: &ColorScheme,
) -> String {
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
        updated_constraint
            .split_once(',')
            .map_or(updated_constraint, |(before, _)| before)
    } else {
        updated_constraint
    };
    let prefix = get_prefix(lower_part);

    let p = palette_for(color_scheme);
    let colored_version = match diff_idx {
        0 => {
            let (r, g, b) = p.major;
            latest.truecolor(r, g, b).to_string()
        }
        1 => {
            let (r, g, b) = p.minor;
            let plain = format!("{}.", new_parts[0]);
            let colored = new_parts[1..].join(".").truecolor(r, g, b).to_string();
            format!("{}{}", plain, colored)
        }
        2 => {
            let (r, g, b) = p.patch;
            let plain = format!("{}.{}.", new_parts[0], new_parts[1]);
            let colored = new_parts[2..].join(".").truecolor(r, g, b).to_string();
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
    use crate::cli::ColorScheme;
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
    fn test_print_color_scheme_preview_does_not_panic() {
        print_color_scheme_preview();
    }

    #[test]
    fn test_print_table_empty() {
        print_table(&[], false, &ColorScheme::Default);
        print_table(&[], true, &ColorScheme::Default);
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
        for scheme in &[
            ColorScheme::Default,
            ColorScheme::OkabeIto,
            ColorScheme::TrafficLight,
            ColorScheme::Severity,
            ColorScheme::HighContrast,
        ] {
            print_table(&updates, true, scheme);
        }
    }

    #[test]
    fn test_print_table_single_package_summary() {
        let updates = vec![make_update(
            "fastapi",
            ">=0.109.0",
            "0.135.1",
            ">=0.135.1",
            BumpKind::Minor,
        )];
        print_table(&updates, true, &ColorScheme::Default);
    }

    #[test]
    fn test_color_updated_constraint_major() {
        let result = color_updated_constraint(">=1.0.0", "2.0.0", ">=2.0.0", &ColorScheme::Default);
        assert!(result.contains("2.0.0"));
        assert!(result.contains(">="));
    }

    #[test]
    fn test_color_updated_constraint_minor() {
        let result =
            color_updated_constraint(">=0.109.0", "0.110.0", ">=0.110.0", &ColorScheme::Default);
        assert!(result.contains("0."));
        assert!(result.contains("110"));
    }

    #[test]
    fn test_color_updated_constraint_patch() {
        let result = color_updated_constraint(">=1.0.0", "1.0.1", ">=1.0.1", &ColorScheme::Default);
        assert!(result.contains("1.0."));
    }

    #[test]
    fn test_color_updated_constraint_compound() {
        let result = color_updated_constraint(
            ">=0.7.3,<0.8.0",
            "1.0.0",
            ">=1.0.0,<2.0.0",
            &ColorScheme::Default,
        );
        assert!(result.contains("1.0.0"));
        assert!(result.contains("2.0.0"));
    }

    #[test]
    fn test_color_updated_constraint_no_base_version() {
        let result = color_updated_constraint("*", "1.0.0", "*", &ColorScheme::Default);
        assert_eq!(result, "*");
    }

    #[test]
    fn test_color_updated_constraint_four_component_version() {
        let result =
            color_updated_constraint(">=1.0.0.0", "1.0.0.1", ">=1.0.0.1", &ColorScheme::Default);
        assert!(result.contains("1.0.0.1"));
        assert!(result.contains(">="));
    }

    #[test]
    fn test_all_schemes_produce_output() {
        // Verify every scheme runs without panic for all bump levels
        let cases = [
            ("1.0.0", "2.0.0", "2.0.0"),
            ("1.0.0", "1.1.0", "1.1.0"),
            ("1.0.0", "1.0.1", "1.0.1"),
        ];
        for scheme in &[
            ColorScheme::Default,
            ColorScheme::OkabeIto,
            ColorScheme::TrafficLight,
            ColorScheme::Severity,
            ColorScheme::HighContrast,
        ] {
            for (cur, _lat, upd) in &cases {
                let result = color_updated_constraint(cur, _lat, upd, scheme);
                assert!(!result.is_empty());
            }
        }
    }
}
