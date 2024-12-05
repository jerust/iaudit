use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use qdrant_client::Qdrant;
use reqwest::Client;
use serde_json::json;
use tokio::fs;
use tokio::sync::Mutex;

use crate::blunder::document::DocumentError;
use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::domain::request::document::generally::Extension;
use crate::domain::request::document::thinktank::UploadDomainRequest;
use crate::helper::proxy;

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
        .with_context(|| format!("Failed to save document of {:?}", filepath))?;

    let absolute = tokio::fs::canonicalize(filepath.as_path())
        .await
        .with_context(|| format!("Failed to get absolute of {:?}", filepath))?;

    let converted = document_convertor(client, absolute, extension.as_ref(), itools)
        .await
        .with_context(|| format!("Failed to run document convertor of {:?}", filepath))?;

    let extracted = document_extractor(client, converted, extension.as_ref(), itools)
        .await
        .with_context(|| format!("Failed to run document extractor of {:?}", filepath))?;
    println!("{}", extracted);

    Ok(())
}

pub async fn document_convertor(
    client: &Client,
    filepath: PathBuf,
    extension: &Extension,
    itools: &ItoolsSettings,
) -> Result<PathBuf, anyhow::Error> {
    if !matches!(extension, Extension::Doc) {
        return Ok(filepath);
    }
    proxy::document_convertor(
        client,
        &itools.word_to_pdf_proxy(),
        json!({"filepath": filepath}),
    )
    .await?;
    Ok(filepath.with_extension("pdf"))
}

pub async fn document_extractor(
    client: &Client,
    filepath: PathBuf,
    extension: &Extension,
    itools: &ItoolsSettings,
) -> Result<String, anyhow::Error> {
    let (proxy, value) = match extension {
        Extension::Xls | Extension::Xlsx => (
            itools.xlsx_reader_proxy(),
            json!({"filepath": filepath, "readmode": "table", "sheet": ""}),
        ),
        Extension::Doc | Extension::Pdf => {
            (itools.pdfx_reader_proxy(), json!({"filepath": filepath}))
        }
        Extension::Docx => (itools.docx_reader_proxy(), json!({"filepath": filepath})),
    };
    proxy::document_extractor(client, &proxy, value).await
}

pub async fn document_splitting() {}

// 文档转换
// 读文件内容
// 不同类型文件不同切片方式
// 切片的优先级
// 构造元数据
// 写入向量库
// 写入磁盘
//
// use std::collections::HashMap;
// fn _parse_json(
//     json_str: &str,
// ) -> Result<HashMap<String, Vec<HashMap<String, String>>>, serde_json::Error> {
//     // 解析最外层的 JSON 对象
//     let outer: HashMap<String, String> = serde_json::from_str(json_str)?;

//     let mut result = HashMap::new();

//     for (sheet_name, sheet_data) in outer {
//         // 解析内层的 JSON 数组
//         let rows: Vec<HashMap<String, String>> = serde_json::from_str(&sheet_data)?;
//         result.insert(sheet_name, rows);
//     }

//     Ok(result)
// }
