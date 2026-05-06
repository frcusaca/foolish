use axum::{
    Json, Router, routing::post,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use foolish_core::Compiler;

#[derive(Deserialize)]
pub struct EvalRequest {
    pub source: String,
}

#[derive(Serialize)]
pub struct EvalResponse {
    pub brane_id: String,
    pub tree: String,
}

async fn eval_handler(Json(req): Json<EvalRequest>) -> Result<Json<EvalResponse>, StatusCode> {
    let firs = Compiler::compile(&req.source)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let tree = firs
        .first()
        .map(|f| foolish_core::fir_to_json(f).unwrap_or_default())
        .unwrap_or_default();
    Ok(Json(EvalResponse {
        brane_id: uuid::Uuid::new_v4().to_string(),
        tree,
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/eval", post(eval_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("Foolish web server on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
