use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};

use crate::configuration::common::CommonSettings;
use crate::configuration::itools::ItoolsSettings;
use crate::database::qdrant::Point;
use crate::domain::business::document::Document;
use crate::domain::request::document::generally::Extension;
use crate::helper::cipher;
use crate::helper::proxy;
use crate::helper::regular;

// 条款正则: 第一条、第二条
static ARTICLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"第(?:[零一二三四五六七八九十百千万]*\d*)条")
        .expect("Failed to build article regex pattern")
});

// 章节正则: 第一章、第二章
static CHAPTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"第(?:[零一二三四五六七八九十百千万]*\d*)章")
        .expect("Failed to build chapter regex pattern")
});

enum Section {
    Article, // 条款
    Chapter, // 章节
}

#[derive(Debug, Default)]
pub struct Article {
    pub article: String, // 条款文本内容
    pub chapter: String, // 条款所属章节
    pub doctype: String, // 文档所属类型, "table"代表表格文档、"plain"代表文本文档
}

impl Article {
    fn serialize(&self) -> Value {
        json!({
            "article": self.article,
            "chapter": self.chapter,
            "doctype": self.doctype,
        })
    }
}

pub async fn document_convertor(
    filepath: PathBuf,
    extension: &Extension,
    client: Arc<Client>,
    itools: Arc<ItoolsSettings>,
) -> Result<PathBuf, anyhow::Error> {
    if !matches!(extension, Extension::Doc) {
        return Ok(filepath);
    }
    proxy::document_convertor(
        &client,
        &itools.word_to_pdf_proxy(),
        json!({"filepath": filepath}),
    )
    .await?;
    Ok(filepath.with_extension("pdf"))
}

pub async fn document_extractor(
    filepath: PathBuf,
    extension: &Extension,
    client: Arc<Client>,
    itools: Arc<ItoolsSettings>,
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
    proxy::document_extractor(&client, &proxy, value).await
}

pub async fn document_splitting(
    content: String,
    extension: &Extension,
    client: Arc<Client>,
    common: Arc<CommonSettings>,
    itools: Arc<ItoolsSettings>,
) -> Result<Vec<Article>, anyhow::Error> {
    let articles = match extension {
        Extension::Xls | Extension::Xlsx => {
            splitting_table_document(&content, client, common, itools)
                .await
                .with_context(|| format!("Failed to splitting table document"))?
                .into_iter()
                .map(|mut article| {
                    article.doctype = "table".to_string();
                    article
                })
                .collect()
        }
        Extension::Doc | Extension::Docx | Extension::Pdf => {
            splitting_plain_document(&content, client, common, itools)
                .await
                .with_context(|| format!("Failed to splitting plain document"))?
                .into_iter()
                .map(|mut article| {
                    article.doctype = "plain".to_string();
                    article
                })
                .collect()
        }
    };
    Ok(articles)
}

pub async fn document_embedding(
    mut document: Document,
    fileuuid: &str,
    filename: &str,
    filepath: PathBuf,
    articles: Vec<Article>,
    client: Arc<Client>,
    itools: Arc<ItoolsSettings>,
) -> Result<Vec<Point<Value>>, anyhow::Error> {
    let mut points = Vec::with_capacity(articles.len());
    for article in articles {
        document.update(article.serialize());
        document.update(json!({"filepath": filepath}));
        points.push(Point {
            id: cipher::murmurhash64int(format!("{}{}", fileuuid, article.article)),
            vector: proxy::document_embedding(
                &client,
                &itools.embedding_proxy(),
                json!({"content": format!("{}{}", filename, article.article)}),
            )
            .await
            .with_context(|| {
                format!("Failed to call document embedding proxy of {:?}", filepath)
            })?,
            payload: document.value(),
        });
    }
    Ok(points)
}

// 对表格文档进行切片
async fn splitting_table_document(
    _content: &str,
    _client: Arc<Client>,
    _common: Arc<CommonSettings>,
    _itools: Arc<ItoolsSettings>,
) -> Result<Vec<Article>, anyhow::Error> {
    Ok(vec![])
}

// 对文本文档进行切片
async fn splitting_plain_document(
    content: &str,
    client: Arc<Client>,
    common: Arc<CommonSettings>,
    itools: Arc<ItoolsSettings>,
) -> Result<Vec<Article>, anyhow::Error> {
    let content: Arc<str> = Arc::from(content);
    let methods: Vec<Box<dyn Callable<Output = Vec<Article>> + Send>> = vec![
        Box::new(SplittingRegulationDocument(content.clone(), common)),
        Box::new(SplittingRecurrenceDocument(content.clone(), client, itools)),
    ];
    try_methods(methods).await
}

#[async_trait]
trait Callable {
    type Output;

    async fn call(&self) -> Result<Self::Output, anyhow::Error>;
}

struct SplittingRegulationDocument(Arc<str>, Arc<CommonSettings>);

#[async_trait]
impl Callable for SplittingRegulationDocument {
    type Output = Vec<Article>;

    async fn call(&self) -> Result<Self::Output, anyhow::Error> {
        splitting_regulation_document(&self.0, self.1.lower_threshold)
            .await
            .with_context(|| "按照法律法规文件的模式来切片失败")
    }
}

// 按照法律法规文件的模式来切片
async fn splitting_regulation_document(
    content: &str,
    threshold: usize,
) -> Result<Vec<Article>, anyhow::Error> {
    let mut sections: Vec<(usize, Section)> = vec![];
    let articles = splitting_document_by_article(content);
    let chapters = splitting_document_by_chapter(content);
    sections.extend(articles.into_iter().map(|(idx, _)| (idx, Section::Article)));
    sections.extend(chapters.into_iter().map(|(idx, _)| (idx, Section::Chapter)));
    if sections.len() < threshold {
        return Err(anyhow::anyhow!(format!("章节条款总数少于{}个", threshold)));
    }
    sections.sort_by(|a, b| a.0.cmp(&b.0));
    let articles = construct_document_by_article(content, sections);
    Ok(articles)
}

// 按照条款对文档进行切片
fn splitting_document_by_article(content: &str) -> Vec<(usize, String)> {
    regular::match_string_with_offset(&ARTICLE_REGEX, content)
}

// 按照章节对文档进行切片
fn splitting_document_by_chapter(content: &str) -> Vec<(usize, String)> {
    regular::match_string_with_offset(&CHAPTER_REGEX, content)
}

// 根据条款对象来构造文档
fn construct_document_by_article(content: &str, sections: Vec<(usize, Section)>) -> Vec<Article> {
    let mut chapter = "";
    let mut articles = vec![];
    let mut iterator = sections.iter().peekable();
    while let Some((i, section)) = iterator.next() {
        if let Some((j, _)) = iterator.peek() {
            let article = content[*i..*j].trim();
            match section {
                Section::Chapter => {
                    chapter = article;
                }
                Section::Article => {
                    articles.push(Article {
                        article: article.to_owned(),
                        chapter: chapter.to_owned(),
                        ..Default::default()
                    });
                }
            }
        } else {
            if let Section::Article = section {
                articles.push(Article {
                    article: content[*i..].trim().to_owned(),
                    chapter: chapter.to_owned(),
                    ..Default::default()
                });
            };
        }
    }
    articles
}

struct SplittingRecurrenceDocument(Arc<str>, Arc<Client>, Arc<ItoolsSettings>);

#[async_trait]
impl Callable for SplittingRecurrenceDocument {
    type Output = Vec<Article>;

    async fn call(&self) -> Result<Self::Output, anyhow::Error> {
        splitting_recurrence_document(&self.0, &self.1, &self.2)
            .await
            .with_context(|| "按照自定义递归规则的模式来进行切片失败")
    }
}

// 按照自定义递归规则模式来切片
async fn splitting_recurrence_document(
    content: &str,
    client: &Client,
    itools: &ItoolsSettings,
) -> Result<Vec<Article>, anyhow::Error> {
    let articles = proxy::document_splitting(
        client,
        itools.splitting_proxy().as_str(),
        json!({"content": content}),
    )
    .await?
    .into_iter()
    .map(|article| Article {
        article,
        ..Default::default()
    })
    .collect();
    Ok(articles)
}

async fn try_methods<T>(
    methods: Vec<Box<dyn Callable<Output = T> + Send>>,
) -> Result<T, anyhow::Error>
where
    T: 'static + Send,
{
    let mut errors = vec![];
    for method in methods {
        match method.call().await {
            Ok(t) => return Ok(t),
            Err(e) => errors.push(e),
        }
    }
    // 只有当所有切片方法全部失败时才算失败
    Err(anyhow::anyhow!(format_errors(errors)))
}

fn format_errors(errors: Vec<anyhow::Error>) -> String {
    errors
        .into_iter()
        .map(|error| {
            error
                .chain()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n\tCaused by:\n\t\t")
        })
        .collect::<Vec<_>>()
        .join("\n\n\t")
}

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
