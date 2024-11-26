use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommonSettings {
    pub thinktank_cache: String,
    pub guideline_cache: String,
}
