use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommonSettings {
    pub lower_threshold: usize,
    pub thinktank_cache: String,
    pub thinktank_space: String,
    pub guideline_cache: String,
    pub guideline_space: String,
}
