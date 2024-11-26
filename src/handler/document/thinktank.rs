use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use actix_web::web::Data;
// use actix_web::web::Path;
use actix_web::Responder;
use anyhow::Context;
use qdrant_client::Qdrant;
use tokio::sync::Mutex;
use tokio::task;
use tracing::Instrument;

use crate::blunder::document::DocumentError;
use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::dto::request::document::thinktank::UploadRequest;
use crate::service::document::thinktank;

#[tracing::instrument(
    name = "上传审计智库文档",
    skip(form, qdrant, itools, common),
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
) -> Result<impl Responder, DocumentError> {
    let domain = form
        .into_inner()
        .try_into()
        .map_err(DocumentError::ValidationError)?;
    task::spawn(
        async move {
            if let Err(error) = thinktank::upload(domain, &qdrant, &itools, &common)
                .await
                .context("后台上传审计智库文档失败")
            {
                // 对于通过spawn创建的后台新任务, 需要显式调用chain方法来追踪错误链路
                for (i, cause) in error.chain().enumerate() {
                    tracing::error!(error = cause, "Error #{} in error chain", i);
                }
            }
        }
        // 将当前跨度传递给新任务, 这样可以让任务继承当前上下文的tracing信息
        .instrument(tracing::Span::current()),
    );
    Ok("rust")
}
