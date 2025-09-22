use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenderedPdfCacheItem {
    pub id: i64,
    pub statement_version_id: i64,
    pub language_id: Option<i64>, // NULL for original statement
    pub pdf_hash: String,
}
