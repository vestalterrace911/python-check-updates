use serde::Deserialize;

#[derive(Deserialize)]
pub struct PypiResponse {
    pub info: PypiInfo,
}

#[derive(Deserialize)]
pub struct PypiInfo {
    pub version: String,
}
