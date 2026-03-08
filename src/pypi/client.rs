use std::time::Duration;

use anyhow::{Result, bail};

use crate::pypi::models::PypiResponse;

/// Per-request timeout for PyPI API calls.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct PypiClient {
    client: reqwest::Client,
    base_url: String,
}

impl PypiClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!(
                "pycu/",
                env!("CARGO_PKG_VERSION"),
                " (https://github.com/Logic-py/python-check-updates)"
            ))
            .timeout(REQUEST_TIMEOUT)
            .https_only(true)
            .build()?;
        Ok(Self {
            client,
            base_url: "https://pypi.org".to_string(),
        })
    }

    /// Test-only constructor pointing at a local mock server (HTTP, no https_only).
    #[cfg(test)]
    pub(crate) fn with_base_url(base_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("pycu/", env!("CARGO_PKG_VERSION")))
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    /// Expose the inner reqwest::Client (e.g. for self-update which makes its own requests).
    pub fn into_inner(self) -> reqwest::Client {
        self.client
    }

    pub async fn get_latest_version(&self, package: &str) -> Result<String> {
        // Validate package name: PEP 508 allows only ASCII letters, digits, '-', '_', '.'
        if package.is_empty()
            || !package
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            bail!("invalid package name: {:?}", package);
        }

        let url = format!("{}/pypi/{}/json", self.base_url, package);
        let resp: PypiResponse = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(resp.info.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_invalid_package_name_empty() {
        let result = PypiClient::new().unwrap().get_latest_version("").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid package name")
        );
    }

    #[tokio::test]
    async fn test_invalid_package_name_slash() {
        let result = PypiClient::new()
            .unwrap()
            .get_latest_version("foo/bar")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_package_name_space() {
        let result = PypiClient::new()
            .unwrap()
            .get_latest_version("foo bar")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_package_name_at_sign() {
        let result = PypiClient::new()
            .unwrap()
            .get_latest_version("foo@bar")
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_package_names_pass_validation() {
        // These should NOT trigger the name guard (they pass the char check).
        // We test the guard logic directly without making network calls.
        for name in &["requests", "my-package", "my_package", "pkg.v2"] {
            assert!(
                !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')),
                "Expected {name} to be a valid package name"
            );
        }
    }

    #[test]
    fn test_into_inner() {
        let _ = PypiClient::new().unwrap().into_inner();
    }

    #[tokio::test]
    async fn test_get_latest_version_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "info": { "version": "2.31.0" }
            })))
            .mount(&server)
            .await;

        let client = PypiClient::with_base_url(&server.uri()).unwrap();
        let version = client.get_latest_version("requests").await.unwrap();
        assert_eq!(version, "2.31.0");
    }

    #[tokio::test]
    async fn test_get_latest_version_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pypi/no-such-package/json"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let client = PypiClient::with_base_url(&server.uri()).unwrap();
        let result = client.get_latest_version("no-such-package").await;
        assert!(result.is_err());
    }
}
