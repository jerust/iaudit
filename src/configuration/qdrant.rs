use qdrant_client::{Qdrant, QdrantError};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QdrantSettings {
    pub host: String,
    pub port: u16,
}

impl QdrantSettings {
    pub fn get_qdrant_client(&self) -> Result<Qdrant, QdrantError> {
        Qdrant::from_url(format!("http://{}:{}", self.host, self.port).as_str()).build()
    }
}
