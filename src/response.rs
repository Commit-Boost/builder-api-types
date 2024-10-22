use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VersionedResponse<T> {
    pub version: Version,
    pub data: T,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum Version {
    #[serde(rename = "deneb")]
    #[default]
    Deneb,
}
