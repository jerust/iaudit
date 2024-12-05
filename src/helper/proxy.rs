use anyhow::{ensure, Context, Error};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;

async fn request_handler(client: &Client, proxy: &str, value: Value) -> Result<Response, Error> {
    let response = client
        .post(proxy)
        .json(&value)
        .send()
        .await
        .with_context(|| format!("Failed to call proxy of {}", proxy))?;
    Ok(response)
}

fn response_status_is_success(response: &Response) -> Result<(), Error> {
    ensure!(
        response.status().is_success(),
        "Failed to fetch {}, status: {}, headers: {:?}",
        response.url(),
        response.status(),
        response.headers(),
    );
    Ok(())
}

pub async fn document_convertor(client: &Client, proxy: &str, value: Value) -> Result<(), Error> {
    let response = request_handler(client, proxy, value).await?;
    response_status_is_success(&response)?;
    Ok(())
}

#[derive(Deserialize)]
struct DocumentExtractor {
    content: String,
}

pub async fn document_extractor(
    client: &Client,
    proxy: &str,
    value: Value,
) -> Result<String, Error> {
    let response = request_handler(client, proxy, value).await?;
    response_status_is_success(&response)?;
    let document_extractor = response
        .json::<DocumentExtractor>()
        .await
        .with_context(|| format!("Failed to deserialize response body of {}", proxy))?;
    Ok(document_extractor.content)
}

#[derive(Deserialize)]
struct DocumentEmbedding {
    vector: Vec<f32>,
}

pub async fn document_embedding(
    client: &Client,
    proxy: &str,
    value: Value,
) -> Result<Vec<f32>, Error> {
    let response = request_handler(client, proxy, value).await?;
    response_status_is_success(&response)?;
    let document_embedding = response
        .json::<DocumentEmbedding>()
        .await
        .with_context(|| format!("Failed to deserialize response body of {}", proxy))?;
    Ok(document_embedding.vector)
}

#[derive(Deserialize)]
struct DocumentReranking {
    scores: Vec<f32>,
}

pub async fn document_reranking(
    client: &Client,
    proxy: &str,
    value: Value,
) -> Result<Vec<f32>, Error> {
    let response = request_handler(client, proxy, value).await?;
    response_status_is_success(&response)?;
    let document_reranking = response
        .json::<DocumentReranking>()
        .await
        .with_context(|| format!("Failed to deserialize response body of {}", proxy))?;
    Ok(document_reranking.scores)
}

#[derive(Deserialize)]
struct DocumentSplitting {
    slices: Vec<String>,
}

pub async fn document_splitting(
    client: &Client,
    proxy: &str,
    value: Value,
) -> Result<Vec<String>, Error> {
    let response = request_handler(client, proxy, value).await?;
    response_status_is_success(&response)?;
    let document_splitting = response
        .json::<DocumentSplitting>()
        .await
        .with_context(|| format!("Failed to deserialize response body of {}", proxy))?;
    Ok(document_splitting.slices)
}
