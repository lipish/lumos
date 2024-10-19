use anyhow::Result;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};

use crate::service::send;
use crate::structs::config::Provider;
use crate::structs::ollama::Message;

pub async fn dispatch(
    model: &str,
    messages: Vec<Message>,
    provider: &Provider,
) -> Result<impl IntoResponse, anyhow::Error> {
    // Send request to the provider service and get the stream
    let stream = send(model, messages, provider).await?;

    // Convert the stream to a Body
    let body = Body::from_stream(stream);

    // Construct the response
    let response = Response::builder()
        .header("Content-Type", "text/plain")
        .body(body)
        .unwrap();

    Ok(response)
}
