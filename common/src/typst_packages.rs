use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct TypstPackagePayload {
    pub namespace: String,
    pub name: String,
    pub version: String,
}
