use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use actix_web::web::Data;
use actix_web::Responder;
use qdrant_client::Qdrant;
use reqwest::Client;
use tokio::sync::Mutex;
use tracing::Instrument;

use crate::blunder::document::DocumentError;
use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::dto::request::document::thinktank::UploadRequest;
use crate::service::document::thinktank;

#[tracing::instrument(
    name = "Upload audit thinktank document",
    skip(form, qdrant, itools, common, client),
    fields(
        fileuuid=%form.uuid.as_str(),
        filename=%form.name.as_str(),
        filearea=%form.range.as_str(),
    )
)]
pub async fn upload(
    form: MultipartForm<UploadRequest>,
    qdrant: Data<Arc<Mutex<Qdrant>>>,
    itools: Data<ItoolsSettings>,
    common: Data<CommonSettings>,
    client: Data<Client>,
) -> Result<impl Responder, DocumentError> {
    let domain = form
        .into_inner()
        .try_into()
        .map_err(DocumentError::ValidationError)?;

    tokio::spawn(
        async move {
            thinktank::upload(domain, &qdrant, &itools, &common, &client)
                .await
                .map_err(|error| {
                    // error = %error, 只会记录最顶层错误
                    // error = ?error, 会记录完整的错误链
                    tracing::error!(error = ?error);
                })
        }
        // 创建独立的任务上下文, 并将跨度传递给新任务, 这样可以让任务继承当前上下文的tracing信息
        .instrument(tracing::info_span!("Upload audit thinktank document task")),
    );

    Ok("rust")
}
