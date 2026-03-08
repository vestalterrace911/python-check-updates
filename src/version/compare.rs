use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};

use crate::parsers::Dependency;
use crate::pypi::client::PypiClient;
use crate::version::constraint::{extract_base_version, update_constraint};

#[derive(Debug, Clone, PartialEq)]
pub enum BumpKind {
    Patch,
    Minor,
    Major,
}

#[derive(Debug, Clone)]
pub struct Update {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub updated_constraint: String,
    pub bump_kind: BumpKind,
}

pub async fn find_updates(
    deps: Vec<Dependency>,
    client: PypiClient,
    concurrency: usize,
) -> Result<Vec<Update>> {
    let total = deps.len() as u64;

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len}  {msg}")?
            .progress_chars("█▉▊▋▌▍▎▏ ")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", ""]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message("starting...");

    let client = Arc::new(client);

    // Carry the original index so we can restore file order after concurrent fetches
    let mut results: Vec<(usize, Option<Update>)> =
        futures::stream::iter(deps.into_iter().enumerate())
            .map(|(idx, dep)| {
                let client = Arc::clone(&client);
                let pb = pb.clone();
                async move {
                    pb.set_message(dep.name.clone());
                    let result = check_dep(&client, &dep).await;
                    pb.inc(1);
                    let update = match result {
                        Ok(u) => u,
                        Err(e) => {
                            pb.println(format!("  warning: {} - {}", dep.name, e));
                            None
                        }
                    };
                    (idx, update)
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

    pb.finish_and_clear();
    eprintln!(
        "Checked {} package{}.",
        total,
        if total == 1 { "" } else { "s" }
    );

    // Restore the original file order
    results.sort_by_key(|(idx, _)| *idx);
    Ok(results.into_iter().filter_map(|(_, u)| u).collect())
}

async fn check_dep(client: &PypiClient, dep: &Dependency) -> Result<Option<Update>> {
    let constraint = dep.constraint.trim();

    // Skip bare wildcard or empty - no meaningful version to compare
    if constraint.is_empty() || constraint == "*" {
        return Ok(None);
    }

    let base = match extract_base_version(constraint) {
        Some(v) => v,
        None => return Ok(None),
    };

    let latest = client.get_latest_version(&dep.name).await?;

    if is_newer(&latest, &base) {
        let updated = update_constraint(constraint, &latest);
        let bump_kind = classify_bump(&base, &latest);
        Ok(Some(Update {
            name: dep.name.clone(),
            current: constraint.to_string(),
            latest,
            updated_constraint: updated,
            bump_kind,
        }))
    } else {
        Ok(None)
    }
}

/// Classify how much the version changed between `old_base` and `latest`.
pub fn classify_bump(old_base: &str, latest: &str) -> BumpKind {
    let old: Vec<u64> = old_base.split('.').filter_map(|p| p.parse().ok()).collect();
    let new: Vec<u64> = latest.split('.').filter_map(|p| p.parse().ok()).collect();

    let old_major = old.first().copied().unwrap_or(0);
    let new_major = new.first().copied().unwrap_or(0);
    if new_major != old_major {
        return BumpKind::Major;
    }

    let old_minor = old.get(1).copied().unwrap_or(0);
    let new_minor = new.get(1).copied().unwrap_or(0);
    if new_minor != old_minor {
        return BumpKind::Minor;
    }

    BumpKind::Patch
}

/// Returns true if `latest` is a strictly newer PEP 440 version than `current`.
pub fn is_newer(latest: &str, current: &str) -> bool {
    use pep440_rs::Version;
    use std::str::FromStr;

    let latest = match Version::from_str(latest) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let current = match Version::from_str(current) {
        Ok(v) => v,
        Err(_) => return false,
    };

    latest > current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.110.0", "0.109.0"));
        assert!(is_newer("2.6.0", "1.10.0"));
        assert!(!is_newer("0.109.0", "0.109.0"));
        assert!(!is_newer("0.108.0", "0.109.0"));
    }

    #[test]
    fn test_is_newer_pre_release() {
        assert!(is_newer("1.0.0", "1.0.0a1"));
        assert!(!is_newer("1.0.0a1", "1.0.0"));
    }

    #[test]
    fn test_is_newer_invalid_version() {
        assert!(!is_newer("not-a-version", "1.0.0"));
        assert!(!is_newer("1.0.0", "not-a-version"));
    }

    #[test]
    fn test_classify_bump_major() {
        assert_eq!(classify_bump("1.0.0", "2.0.0"), BumpKind::Major);
        assert_eq!(classify_bump("0.7.3", "1.0.0"), BumpKind::Major);
    }

    #[test]
    fn test_classify_bump_minor() {
        assert_eq!(classify_bump("1.0.0", "1.1.0"), BumpKind::Minor);
        assert_eq!(classify_bump("0.109.0", "0.110.0"), BumpKind::Minor);
        assert_eq!(classify_bump("0.7.3", "0.8.0"), BumpKind::Minor);
    }

    #[test]
    fn test_classify_bump_patch() {
        assert_eq!(classify_bump("1.0.0", "1.0.1"), BumpKind::Patch);
        assert_eq!(classify_bump("7.3.0", "7.3.1"), BumpKind::Patch);
    }

    // check_dep and find_updates - network paths covered via wiremock.

    #[tokio::test]
    async fn test_check_dep_update_available() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pypi/fastapi/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "info": { "version": "0.135.1" }
            })))
            .mount(&server)
            .await;

        let client = PypiClient::with_base_url(&server.uri()).unwrap();
        let dep = crate::parsers::Dependency {
            name: "fastapi".to_string(),
            constraint: ">=0.109.0".to_string(),
        };
        let update = check_dep(&client, &dep).await.unwrap().unwrap();
        assert_eq!(update.name, "fastapi");
        assert_eq!(update.latest, "0.135.1");
        assert_eq!(update.bump_kind, BumpKind::Minor);
        assert_eq!(update.updated_constraint, ">=0.135.1");
    }

    #[tokio::test]
    async fn test_check_dep_already_up_to_date() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "info": { "version": "2.28.0" }
            })))
            .mount(&server)
            .await;

        let client = PypiClient::with_base_url(&server.uri()).unwrap();
        let dep = crate::parsers::Dependency {
            name: "requests".to_string(),
            constraint: ">=2.28.0".to_string(),
        };
        assert!(check_dep(&client, &dep).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_find_updates_returns_update() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "info": { "version": "2.31.0" }
            })))
            .mount(&server)
            .await;

        let client = PypiClient::with_base_url(&server.uri()).unwrap();
        let deps = vec![crate::parsers::Dependency {
            name: "requests".to_string(),
            constraint: ">=2.28.0".to_string(),
        }];
        let updates = find_updates(deps, client, 1).await.unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].name, "requests");
        assert_eq!(updates[0].latest, "2.31.0");
    }

    #[tokio::test]
    async fn test_find_updates_skips_failed_package() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pypi/bad-pkg/json"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = PypiClient::with_base_url(&server.uri()).unwrap();
        let deps = vec![crate::parsers::Dependency {
            name: "bad-pkg".to_string(),
            constraint: ">=1.0.0".to_string(),
        }];
        // Failed packages are warned and skipped, not propagated as errors.
        let updates = find_updates(deps, client, 1).await.unwrap();
        assert!(updates.is_empty());
    }

    // check_dep early-return paths - no network calls made.

    #[tokio::test]
    async fn test_check_dep_skips_wildcard() {
        let client = PypiClient::new().unwrap();
        let dep = crate::parsers::Dependency {
            name: "any".to_string(),
            constraint: "*".to_string(),
        };
        assert!(check_dep(&client, &dep).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_check_dep_skips_empty_constraint() {
        let client = PypiClient::new().unwrap();
        let dep = crate::parsers::Dependency {
            name: "any".to_string(),
            constraint: String::new(),
        };
        assert!(check_dep(&client, &dep).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_check_dep_skips_bare_operator() {
        // ">=" strips to "" → extract_base_version returns None → Ok(None)
        let client = PypiClient::new().unwrap();
        let dep = crate::parsers::Dependency {
            name: "any".to_string(),
            constraint: ">=".to_string(),
        };
        assert!(check_dep(&client, &dep).await.unwrap().is_none());
    }
}
