/// Extract the base version number from a constraint string, stripping any operator prefix.
/// Returns `None` for empty, `*`, or unparseable constraints.
///
/// Examples:
/// - `">=1.2.3"` → `Some("1.2.3")`
/// - `"^0.109.0"` → `Some("0.109.0")`
/// - `"~=1.4"` → `Some("1.4")`
/// - `"*"` → `None`
pub fn extract_base_version(constraint: &str) -> Option<String> {
    let s = constraint.trim();

    if s.is_empty() || s == "*" {
        return None;
    }

    // Compound constraint: take the first component for comparison
    let s = if s.contains(',') {
        s.split_once(',').map_or(s, |(before, _)| before).trim()
    } else {
        s
    };

    // Strip operator prefix (order matters: longer operators first)
    let version = s
        .trim_start_matches("===")
        .trim_start_matches("~=")
        .trim_start_matches("!=")
        .trim_start_matches(">=")
        .trim_start_matches("<=")
        .trim_start_matches("==")
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim();

    if version.is_empty() || version == "*" {
        return None;
    }

    // Strip trailing wildcard (e.g. "1.2.*" → "1.2")
    let version = version.trim_end_matches(".*").trim_end_matches('.');

    Some(version.to_string())
}

/// Return the operator prefix of a constraint string.
///
/// Examples: `">=1.0"` → `">="`, `"^1.2"` → `"^"`, `"1.0"` → `""`
pub fn get_prefix(constraint: &str) -> &str {
    let s = constraint.trim();
    // Check longer operators first
    for prefix in &["===", "~=", "!=", ">=", "<=", "==", ">", "<", "^", "~"] {
        if s.starts_with(prefix) {
            return prefix;
        }
    }
    ""
}

/// Build an updated constraint preserving the original operator and, for compound
/// constraints (`>=old,<bound`), intelligently updating the upper bound:
///
/// - If `new_version` fits within the existing upper bound → only the lower part is bumped.
/// - If `new_version` meets or exceeds the upper bound → both bounds are updated,
///   keeping the same granularity style:
///   - `<X.0.0` (major boundary) → `<(new_major+1).0.0`
///   - `<X.Y.0` (minor boundary) → `<X.(new_minor+1).0`
///   - `<X.Y.Z` (patch boundary) → `<X.Y.(new_patch+1)`
pub fn update_constraint(constraint: &str, new_version: &str) -> String {
    let s = constraint.trim();

    if s.is_empty() || s == "*" {
        return s.to_string();
    }

    if s.contains(',') {
        return update_compound_constraint(s, new_version);
    }

    let prefix = get_prefix(s);
    format!("{}{}", prefix, new_version)
}

fn update_compound_constraint(constraint: &str, new_version: &str) -> String {
    // Find the upper bound component (starts with '<')
    let upper_str = constraint
        .split(',')
        .map(str::trim)
        .find(|p| p.starts_with('<'));

    let upper_str = match upper_str {
        None => return format!(">={}", new_version),
        Some(u) => u,
    };

    // Strip '<' or '<=' to get the bare upper version
    let upper_ver = upper_str
        .trim_start_matches("<=")
        .trim_start_matches('<')
        .trim();

    // If new version is still within the existing upper bound, just bump the lower part
    if !version_gte(new_version, upper_ver) {
        return format!(">={},{}", new_version, upper_str);
    }

    // New version meets or exceeds upper bound - bump the upper bound too
    let upper_parts: Vec<u64> = upper_ver
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    let new_parts: Vec<u64> = new_version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

    let new_upper = compute_new_upper(&upper_parts, &new_parts);
    format!(">={},{}", new_version, new_upper)
}

/// `true` if version string `a` >= version string `b` (component-wise u64 comparison).
fn version_gte(a: &str, b: &str) -> bool {
    let av: Vec<u64> = a.split('.').filter_map(|p| p.parse().ok()).collect();
    let bv: Vec<u64> = b.split('.').filter_map(|p| p.parse().ok()).collect();
    let len = av.len().max(bv.len());
    for i in 0..len {
        let ai = av.get(i).copied().unwrap_or(0);
        let bi = bv.get(i).copied().unwrap_or(0);
        match ai.cmp(&bi) {
            std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Equal => {}
        }
    }
    true // equal counts as >=
}

/// Compute a new `<upper>` string by bumping the appropriate component of `new_version`,
/// matching the granularity of the old upper bound.
fn compute_new_upper(old_upper: &[u64], new_ver: &[u64]) -> String {
    let old_patch = old_upper.get(2).copied().unwrap_or(0);
    let old_minor = old_upper.get(1).copied().unwrap_or(0);

    let new_major = new_ver.first().copied().unwrap_or(0);
    let new_minor = new_ver.get(1).copied().unwrap_or(0);
    let new_patch = new_ver.get(2).copied().unwrap_or(0);

    if old_patch != 0 {
        // Patch-level boundary: <X.Y.Z → <X.Y.(new_patch+1)
        format!("<{}.{}.{}", new_major, new_minor, new_patch + 1)
    } else if old_minor != 0 {
        // Minor-level boundary: <X.Y.0 → <X.(new_minor+1).0
        format!("<{}.{}.0", new_major, new_minor + 1)
    } else {
        // Major-level boundary: <X.0.0 → <(new_major+1).0.0
        format!("<{}.0.0", new_major + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_version() {
        assert_eq!(extract_base_version(">=1.2.3"), Some("1.2.3".to_string()));
        assert_eq!(
            extract_base_version("^0.109.0"),
            Some("0.109.0".to_string())
        );
        assert_eq!(extract_base_version("~=1.4"), Some("1.4".to_string()));
        assert_eq!(extract_base_version("==2.0.0"), Some("2.0.0".to_string()));
        assert_eq!(extract_base_version("1.0.0"), Some("1.0.0".to_string()));
        assert_eq!(extract_base_version("*"), None);
        assert_eq!(extract_base_version(""), None);
        assert_eq!(extract_base_version(">=1.0,<2.0"), Some("1.0".to_string()));
        assert_eq!(extract_base_version("~7.3.0"), Some("7.3.0".to_string()));
    }

    #[test]
    fn test_get_prefix() {
        assert_eq!(get_prefix(">=1.0"), ">=");
        assert_eq!(get_prefix("^1.2.3"), "^");
        assert_eq!(get_prefix("~=1.4"), "~=");
        assert_eq!(get_prefix("==2.0"), "==");
        assert_eq!(get_prefix("1.0.0"), "");
        assert_eq!(get_prefix("~7.3.0"), "~");
    }

    #[test]
    fn test_update_constraint_simple() {
        assert_eq!(update_constraint(">=0.109.0", "0.110.0"), ">=0.110.0");
        assert_eq!(update_constraint("^1.10.0", "2.6.0"), "^2.6.0");
        assert_eq!(update_constraint("0.1.6", "0.3.0"), "0.3.0");
        assert_eq!(update_constraint("*", "1.0.0"), "*");
    }

    #[test]
    fn test_update_compound_within_bounds() {
        // New version still fits inside the upper bound - only lower is bumped
        assert_eq!(
            update_constraint(">=1.19.1,<2.0.0", "1.20.0"),
            ">=1.20.0,<2.0.0"
        );
        assert_eq!(
            update_constraint(">=4.5.1,<5.0.0", "4.6.0"),
            ">=4.6.0,<5.0.0"
        );
        assert_eq!(
            update_constraint(">=9.0.2,<10.0.0", "9.0.3"),
            ">=9.0.3,<10.0.0"
        );
    }

    #[test]
    fn test_update_compound_exceeds_major_bound() {
        // New version crosses the major boundary → bump both, major-level style
        assert_eq!(
            update_constraint(">=1.19.1,<2.0.0", "2.1.0"),
            ">=2.1.0,<3.0.0"
        );
        assert_eq!(
            update_constraint(">=0.7.3,<1.0.0", "1.2.0"),
            ">=1.2.0,<2.0.0"
        );
    }

    #[test]
    fn test_update_compound_exceeds_minor_bound() {
        // New version crosses a minor boundary → bump both, minor-level style
        assert_eq!(
            update_constraint(">=1.2.0,<1.3.0", "1.4.1"),
            ">=1.4.1,<1.5.0"
        );
    }

    #[test]
    fn test_update_compound_exceeds_patch_bound() {
        // New version crosses a patch boundary → bump both, patch-level style
        assert_eq!(
            update_constraint(">=1.2.3,<1.2.4", "1.2.5"),
            ">=1.2.5,<1.2.6"
        );
    }

    #[test]
    fn test_update_compound_no_upper() {
        // Unusual: compound with no upper bound falls back to >=new
        assert_eq!(update_constraint(">=1.0,!=1.5", "2.0.0"), ">=2.0.0");
    }

    #[test]
    fn test_update_compound_new_version_equals_upper_bound() {
        // new_version == upper_ver triggers the `true // equal counts as >=` path
        // in version_gte, then falls through to compute_new_upper.
        assert_eq!(
            update_constraint(">=1.0.0,<2.0.0", "2.0.0"),
            ">=2.0.0,<3.0.0"
        );
    }
}
