use crate::store::Store;
use axum::{
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use paint_core::{Document, Operation, MAX_BYTES, VERSION};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use subtle::ConstantTimeEq;
use uuid::Uuid;
#[derive(Clone)]
pub struct Api {
    pub store: Arc<Mutex<Store>>,
    pub token: String,
}
#[derive(Debug)]
struct Error(StatusCode, String);
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        let message = e.to_string();
        let status = if message == "REVISION_CONFLICT" {
            StatusCode::CONFLICT
        } else if e
            .downcast_ref::<std::io::Error>()
            .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        Self(
            status,
            if message == "REVISION_CONFLICT" {
                "Document changed. Load the latest revision before saving.".into()
            } else {
                message
            },
        )
    }
}
fn bad(message: &str) -> Error {
    Error(StatusCode::BAD_REQUEST, message.into())
}
fn id(raw: &str) -> Result<Uuid, Error> {
    Uuid::parse_str(raw).map_err(|_| bad("Invalid document UUID"))
}
const ORIGINS: &[&str] = &[
    "http://127.0.0.1:3020",
    "http://localhost:3020",
    "http://127.0.0.1:4173",
    "http://localhost:4173",
    "http://tauri.localhost",
    "https://tauri.localhost",
    "tauri://localhost",
];
async fn guard(State(state): State<Api>, request: Request, next: Next) -> Response {
    let headers = request.headers();
    let host = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let mut parts = host.split(':');
    let name = parts.next().unwrap_or("");
    let port = parts.next();
    if !matches!(name, "127.0.0.1" | "localhost")
        || parts.next().is_some()
        || port.is_some_and(|p| p.parse::<u16>().is_err())
    {
        return Error(StatusCode::FORBIDDEN, "Host not allowed".into()).into_response();
    }
    let origin = headers.get(header::ORIGIN).cloned();
    if origin
        .as_ref()
        .is_some_and(|h| !ORIGINS.iter().any(|v| h == *v))
    {
        return Error(StatusCode::FORBIDDEN, "Origin not allowed".into()).into_response();
    }
    let mut response = if request.method() == axum::http::Method::OPTIONS {
        StatusCode::NO_CONTENT.into_response()
    } else {
        let supplied = headers
            .get(header::AUTHORIZATION)
            .map(|s| s.as_bytes())
            .unwrap_or_default();
        let expected = format!("Bearer {}", state.token);
        if !bool::from(supplied.ct_eq(expected.as_bytes())) {
            return Error(
                StatusCode::UNAUTHORIZED,
                "Valid bearer token required".into(),
            )
            .into_response();
        }
        next.run(request).await
    };
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    if let Some(origin) = origin {
        h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        h.insert(header::VARY, HeaderValue::from_static("Origin"));
    }
    h.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, PUT, OPTIONS"),
    );
    h.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Authorization, Content-Type"),
    );
    response
}
pub fn router(store: Store, token: String) -> Router {
    let state = Api {
        store: Arc::new(Mutex::new(store)),
        token,
    };
    Router::new().route("/health",get(||async{Json(json!({"name":"openaec-paint-studio","version":VERSION,"status":"ok","implementation":"rust"}))}))
 .route("/v1/documents",get(list).post(create)).route("/v1/documents/{id}",get(read).put(replace)).route("/v1/documents/{id}/operations",post(append)).route("/v1/documents/{id}/undo",post(undo)).route("/v1/documents/{id}/export.png",get(export))
 .layer(DefaultBodyLimit::max(MAX_BYTES)).layer(middleware::from_fn_with_state(state.clone(),guard)).with_state(state)
}
async fn list(State(s): State<Api>) -> Result<Json<Value>, Error> {
    Ok(Json(s.store.lock().unwrap().list()?))
}
async fn create(
    State(s): State<Api>,
    Json(d): Json<Document>,
) -> Result<(StatusCode, Json<Value>), Error> {
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(s.store.lock().unwrap().create(d)?).unwrap()),
    ))
}
async fn read(State(s): State<Api>, Path(raw): Path<String>) -> Result<Json<Value>, Error> {
    Ok(Json(
        serde_json::to_value(s.store.lock().unwrap().get(id(&raw)?)?).unwrap(),
    ))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Replace {
    document: Document,
    expected_revision: u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Append {
    operations: Vec<Operation>,
    expected_revision: u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Revision {
    expected_revision: u64,
}
async fn replace(
    State(s): State<Api>,
    Path(raw): Path<String>,
    Json(b): Json<Replace>,
) -> Result<Json<Value>, Error> {
    Ok(Json(
        serde_json::to_value(s.store.lock().unwrap().replace(
            id(&raw)?,
            b.document,
            b.expected_revision,
        )?)
        .unwrap(),
    ))
}
async fn append(
    State(s): State<Api>,
    Path(raw): Path<String>,
    Json(b): Json<Append>,
) -> Result<Json<Value>, Error> {
    Ok(Json(
        serde_json::to_value(s.store.lock().unwrap().append(
            id(&raw)?,
            b.operations,
            b.expected_revision,
        )?)
        .unwrap(),
    ))
}
async fn undo(
    State(s): State<Api>,
    Path(raw): Path<String>,
    Json(b): Json<Revision>,
) -> Result<Json<Value>, Error> {
    Ok(Json(
        serde_json::to_value(
            s.store
                .lock()
                .unwrap()
                .undo(id(&raw)?, b.expected_revision)?,
        )
        .unwrap(),
    ))
}
async fn export(State(s): State<Api>, Path(raw): Path<String>) -> Result<Response, Error> {
    let doc = s.store.lock().unwrap().get(id(&raw)?)?.document;
    let bytes = paint_core::png(&doc)?;
    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"drawing.png\"",
            ),
        ],
        bytes,
    )
        .into_response())
}
