use std::sync::Arc;

use qdrant_client::Qdrant;
use tokio::sync::Mutex;

use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::domain::request::document::thinktank::UploadDomainRequest;

#[tracing::instrument(name = "后台上传审计智库文档", skip(domain, qdrant, itools, common))]
pub async fn upload(
    domain: UploadDomainRequest,
    qdrant: &Arc<Mutex<Qdrant>>,
    itools: &ItoolsSettings,
    common: &CommonSettings,
) -> Result<(), anyhow::Error> {
    Ok(())
}
