// HTTP handlers grouped by area. Also hosts shared response helpers and the
// Askama template definitions used by each handler.
pub mod admin;
pub mod cart;
pub mod checkout;
pub mod setup;
pub mod store;

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};

/// Render any Askama template into an axum `Response`. Centralizes the
/// content-type handling and error fallback so handlers can call
/// `render(TemplateStruct { .. })`.
pub fn render<T: askama::Template>(t: T) -> Response {
    match t.render() {
        Ok(body) => ([(axum::http::header::CONTENT_TYPE, T::MIME_TYPE)], body).into_response(),
        Err(e) => server_error(e),
    }
}

/// Extension trait so template structs can be turned into a response fluently
/// with `.page()` (kept separate from axum's IntoResponse to avoid orphan rules).
pub trait TemplateResponse {
    fn page(self) -> Response;
}

impl<T: askama::Template> TemplateResponse for T {
    fn page(self) -> Response {
        render(self)
    }
}

/// Convert an anyhow error into a 500 response with a friendly message.
pub fn server_error<E: std::fmt::Display>(e: E) -> Response {
    tracing::error!("internal error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(format!(
            "<h1>Terjadi kesalahan</h1><p>Maaf, ada masalah di server. Silakan coba lagi.</p><pre style='color:#888'>{e}</pre>"
        )),
    )
        .into_response()
}

/// Helper to redirect with a path.
pub fn redirect(to: &str) -> Response {
    Redirect::to(to).into_response()
}
