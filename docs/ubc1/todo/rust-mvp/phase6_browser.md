# Phase 6 — Web Brane Browser (LOD)

> Goal: A browser-based viewer for Foolish brane trees, with level-of-detail
> windowed viewports.

---

## Phase 6 Deliverable

A web app in the `foolish-web` crate that:

1. Accepts Foolish source via `POST /eval` and runs it through Phase 1 + Phase 2 + Phase 3 + Phase 5.
2. Returns the resulting brane tree as JSON.
3. Renders viewports showing windows of consecutive statements.
4. Click a nested brane → open a new viewport.
5. Search box drives navigation.

---

## Stack

- **HTTP server**: `axum` + `serde_json` (Rust's standard async web framework,
  integrates trivially with serde FIR codecs)
- **Frontend**: thin HTML+JS or htmx — no heavy SPA needed
- **State**: server-side in-memory `HashMap<String, FirRef>` keyed by brane ID

```toml
# foolish-web/Cargo.toml
[package]
name = "foolish-web"
version = "0.1.0"
edition = "2024"

[dependencies]
foolish-core = { path = "../foolish-core" }
axum = "0.8"
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
```

---

## API Sketch

| Endpoint | Purpose |
|----------|---------|
| `POST /eval` | Compile + evaluate; return brane tree as JSON |
| `GET /brane/:id/window?from=N&size=M` | Windowed view of statements |
| `GET /brane/:id/search?q=pat&dir=backward` | Search within a brane |

---

## Handler Sketch

```rust
use axum::{Router, Json, routing::post};

#[derive(serde::Serialize)]
struct EvalRequest {
    source: String,
}

#[derive(serde::Serialize)]
struct EvalResponse {
    brane_id: String,
    tree: Fir,  // serde serializes our Fir enum
}

async fn eval_handler(Json(req): Json<EvalRequest>) -> Result<Json<EvalResponse>, StatusCode> {
    let fir = Compiler::compile(&req.source)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ubc::run_to_completion(&fir)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(EvalResponse {
        brane_id: Uuid::new_v4().to_string(),
        tree: fir,
    }))
}
```

---

## Phase 6 Exit Criteria

- Viewer renders a 1000-statement brane without jank.
- Multiple simultaneous viewports work.
- Constanic statements visually distinct.
- Click-to-open-nested-brane works without page reload.

---

## Last Updated

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — Rust Phase 6 web browser plan. Uses axum for HTTP,
serde for JSON serialization, tokio for async runtime, uuid for brane IDs.
