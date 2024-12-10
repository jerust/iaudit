use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use qdrant_client::Qdrant;
use reqwest::Client;
use tokio::fs;

use crate::blunder::document::DocumentError;
use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::database::qdrant::QdrantLibrary;
use crate::domain::request::document::thinktank::UploadDomainRequest;
use crate::service::document::pipeliner;

#[tracing::instrument(
    name = "Upload audit thinktank document service",
    skip(domain, qdrant, itools, common)
)]
pub async fn upload(
    domain: UploadDomainRequest,
    client: &Arc<Client>,
    qdrant: &Arc<Qdrant>,
    common: &Arc<CommonSettings>,
    itools: &Arc<ItoolsSettings>,
) -> Result<(), DocumentError> {
    let directory = Path::new(common.thinktank_cache.as_str()).join(domain.uuid.as_str());

    fs::create_dir_all(directory.as_path())
        .await
        .with_context(|| format!("Failed to create directory of {:?}", directory))?;

    let fileuuid = domain.uuid.to_string();
    let filename = domain.name.name().to_string();
    let filepath = directory.join(filename.as_str());
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

    let converted = pipeliner::document_convertor(
        absolute.clone(),
        extension.as_ref(),
        client.clone(),
        itools.clone(),
    )
    .await
    .with_context(|| format!("Failed to run document convertor of {:?}", filepath))?;

    let extracted = pipeliner::document_extractor(
        converted,
        extension.as_ref(),
        client.clone(),
        itools.clone(),
    )
    .await
    .with_context(|| format!("Failed to run document extractor of {:?}", filepath))?;

    let articles = pipeliner::document_splitting(
        extracted,
        extension.as_ref(),
        client.clone(),
        common.clone(),
        itools.clone(),
    )
    .await
    .with_context(|| format!("Failed to run document splitting of {:?}", filepath))?;

    let points = pipeliner::document_embedding(
        domain.into(),
        fileuuid.as_str(),
        filename.as_ref(),
        absolute,
        articles,
        client.clone(),
        itools.clone(),
    )
    .await
    .with_context(|| format!("Failed to run document embedding of {:?}", filepath))?;

    qdrant
        .async_upsert_points(common.thinktank_space.to_owned(), points)
        .await
        .with_context(|| format!("Failed to insert to database of {:?}", filepath))?;

    Ok(())
}
