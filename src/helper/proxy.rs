use std::time::Duration;

use anyhow::{ensure, Context};
use reqwest::{IntoUrl, Response};
use serde::Deserialize;
use serde_json::Value;

// 公共的响应检查函数
fn check_if_response_succeed(response: &Response) -> Result<(), anyhow::Error> {
    ensure!(
        response.status().is_success(),
        "Failed to fetch {}, status: {}, headers: {:?}",
        response.url(),
        response.status(),
        response.headers(),
    );
    Ok(())
}

// 公共的代理请求函数
async fn proxy<Url: IntoUrl>(
    url: Url,
    value: Value,
    duration: Duration,
) -> Result<reqwest::Response, reqwest::Error> {
    reqwest::Client::new()
        .post(url)
        .json(&value)
        .timeout(duration)
        .send()
        .await
}

// 用于代理请求的宏
macro_rules! proxy_request {
    ($url:expr, $value:expr, $duration:expr) => {{
        let response = proxy($url, $value, $duration)
            .await
            .with_context(|| format!("Failed to call proxy of {}", $url))?;
        check_if_response_succeed(&response)?;
        response
    }};
}

macro_rules! regular_macro {
    ($name:ident) => {
        pub async fn $name<S: AsRef<str>>(
            url: S,
            value: Value,
            duration: Duration,
        ) -> Result<(), anyhow::Error> {
            proxy_request!(url.as_ref(), value, duration);
            Ok(())
        }
    };
}

#[derive(Deserialize)]
struct Reading {
    content: String,
}

/// 文档类型转换
pub async fn document_convertor() {}

/// 文档内容提取
pub async fn document_extractor() {}

macro_rules! reading_macro {
    ($name:ident) => {
        pub async fn $name<S: AsRef<str>>(
            url: S,
            value: Value,
            duration: Duration,
        ) -> Result<String, anyhow::Error> {
            let reading = proxy_request!(url.as_ref(), value, duration)
                .json::<Reading>()
                .await
                .with_context(|| {
                    format!("Failed to deserialize response body of {}", url.as_ref())
                })?;
            Ok(reading.content)
        }
    };
}

#[derive(Deserialize)]
struct Embedding {
    vector: Vec<f32>,
}

macro_rules! embedding_macro {
    ($name:ident) => {
        pub async fn $name<S: AsRef<str>>(
            url: S,
            value: Value,
            duration: Duration,
        ) -> Result<Vec<f32>, anyhow::Error> {
            let embedding = proxy_request!(url.as_ref(), value, duration)
                .json::<Embedding>()
                .await
                .with_context(|| {
                    format!("Failed to deserialize response body of {}", url.as_ref())
                })?;
            Ok(embedding.vector)
        }
    };
}

#[derive(Deserialize)]
struct Reranking {
    scores: Vec<f32>,
}

macro_rules! reranking_macro {
    ($name:ident) => {
        pub async fn $name<S: AsRef<str>>(
            url: S,
            value: Value,
            duration: Duration,
        ) -> Result<Vec<f32>, anyhow::Error> {
            let reranking = proxy_request!(url.as_ref(), value, duration)
                .json::<Reranking>()
                .await
                .with_context(|| {
                    format!("Failed to deserialize response body of {}", url.as_ref())
                })?;
            Ok(reranking.scores)
        }
    };
}

#[derive(Deserialize)]
struct Splitting {
    slices: Vec<String>,
}

macro_rules! splitting_macro {
    ($name:ident) => {
        pub async fn $name<S: AsRef<str>>(
            url: S,
            value: Value,
            duration: Duration,
        ) -> Result<Vec<String>, anyhow::Error> {
            let splitting = proxy_request!(url.as_ref(), value, duration)
                .json::<Splitting>()
                .await
                .with_context(|| {
                    format!("Failed to deserialize response body of {}", url.as_ref())
                })?;
            Ok(splitting.slices)
        }
    };
}

regular_macro!(convert_word_to_pdf);
regular_macro!(convert_pdf_to_html);
reading_macro!(reading_excel);
reading_macro!(reading_docx);
reading_macro!(reading_pdf);
embedding_macro!(embedding);
reranking_macro!(reranking);
splitting_macro!(splitting);
