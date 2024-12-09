use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use qdrant_client::Qdrant;
use reqwest::Client;
use tokio::fs;
use tokio::sync::Mutex;

use crate::blunder::document::DocumentError;
use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::domain::request::document::thinktank::UploadDomainRequest;
use crate::service::document::pipeliner;

#[tracing::instrument(
    name = "Upload audit thinktank document service",
    skip(domain, qdrant, itools, common)
)]
pub async fn upload(
    domain: UploadDomainRequest,
    qdrant: &Arc<Mutex<Qdrant>>,
    itools: &ItoolsSettings,
    common: &CommonSettings,
    client: &Client,
) -> Result<(), DocumentError> {
    println!("{}", Arc::strong_count(qdrant));

    let directory = Path::new(common.thinktank_cache.as_str()).join(domain.uuid);

    fs::create_dir_all(directory.as_path())
        .await
        .with_context(|| format!("Failed to create directory of {:?}", directory))?;

    let filename = domain.name.name();
    let filepath = directory.join(filename.as_ref());
    let extension = domain.name.extension();

    // 待办: 文件保存失败则删除目录
    domain
        .file
        .persist(filepath.as_path())
        .await
        .with_context(|| format!("Failed to save document to local of {:?}", filepath))?;

    let absolute = tokio::fs::canonicalize(filepath.as_path())
        .await
        .with_context(|| format!("Failed to get absolute filepath for {:?}", filepath))?;

    let converted = pipeliner::document_convertor(client, absolute, extension.as_ref(), itools)
        .await
        .with_context(|| format!("Failed to run document convertor of {:?}", filepath))?;

    let extracted = pipeliner::document_extractor(client, converted, extension.as_ref(), itools)
        .await
        .with_context(|| format!("Failed to run document extractor of {:?}", filepath))?;
    println!("{}", extracted);

    pipeliner::document_splitting(client, &extracted, extension.as_ref(), itools)
        .await
        .with_context(|| format!("Failed to run document splitting of {:?}", filepath))?;

    Ok(())
}
