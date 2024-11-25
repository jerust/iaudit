use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommonSettings {
    pub thinktank_document_cache: String,
    pub guideline_document_cache: String,
}
