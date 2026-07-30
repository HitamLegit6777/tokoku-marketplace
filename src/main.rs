// TokoKu - platform toko online siap pakai untuk UMKM Indonesia.
// Entry point: sets up DB, session store, routing, and static assets.

// This is a starter kit: a handful of helper functions, model fields, and
// convenience accessors are provided as reusable API surface for people
// extending the template (e.g. `helpers::price`, `themes::all_google_fonts`,
// the `User` model for future multi-admin support). They are intentionally
// kept even when not yet wired into a route, so dead-code is allowed crate-wide.
#![allow(dead_code)]

mod db;
mod filters;
mod handlers;
mod icons;
mod models;
mod services;
mod state;

use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use state::AppState;
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::time::Duration;
use tower_sessions::session_store::ExpiredDeletion;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_rusqlite_store::{tokio_rusqlite, RusqliteStore};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tokoku=info,tower_http=warn".into()),
        )
        .compact()
        .init();

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/tokoku.db".into());
    std::fs::create_dir_all("data").ok();
    std::fs::create_dir_all("static/uploads").ok();

    let db = db::init(&db_path)?;

    // Bootstrap: seed demo data and create default admin if none exists.
    services::seed::seed_if_empty(&db)?;
    ensure_default_admin(&db)?;

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| format!("http://localhost:{port}"));

    let secure_cookie = base_url.starts_with("https://");
    let csrf_origin = origin_from_base_url(&base_url);
    let state = AppState {
        db,
        base_url,
        login_limiter: Default::default(),
    };

    // Keep carts and admin logins across process restarts in a dedicated
    // SQLite database. The maintained store serializes access asynchronously.
    let session_db_path =
        std::env::var("SESSION_DATABASE_PATH").unwrap_or_else(|_| "data/sessions.db".into());
    if let Some(parent) = std::path::Path::new(&session_db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let session_conn = tokio_rusqlite::Connection::open(&session_db_path).await?;
    let session_store = RusqliteStore::new(session_conn);
    session_store.migrate().await?;
    tokio::spawn(
        session_store
            .clone()
            .continuously_delete_expired(std::time::Duration::from_secs(60 * 60)),
    );
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(secure_cookie)
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)))
        .with_name("tokoku_session");

    let app = Router::new()
        // ---------- Storefront ----------
        .route("/", get(handlers::store::home))
        .route("/robots.txt", get(handlers::store::robots_txt))
        .route("/sitemap.xml", get(handlers::store::sitemap_xml))
        .route("/products", get(handlers::store::product_list))
        .route("/product/:slug", get(handlers::store::product_detail))
        .route("/category/:slug", get(handlers::store::category_page))
        .route("/search", get(handlers::store::search))
        .route("/page/:slug", get(handlers::store::content_page))
        .route(
            "/product/:slug/review",
            post(handlers::store::submit_review),
        )
        // ---------- Cart ----------
        .route("/cart", get(handlers::cart::view_cart))
        .route("/cart/add", post(handlers::cart::add_to_cart))
        .route("/cart/update", post(handlers::cart::update_cart))
        .route("/cart/remove", post(handlers::cart::remove_from_cart))
        .route("/cart/count", get(handlers::cart::cart_count_api))
        // ---------- Checkout & orders ----------
        .route("/checkout", get(handlers::checkout::checkout_page))
        .route("/checkout", post(handlers::checkout::place_order))
        .route(
            "/order/:number",
            get(handlers::checkout::order_confirmation),
        )
        .route(
            "/order/:number/proof",
            post(handlers::checkout::upload_proof),
        )
        .route("/track", get(handlers::checkout::track_form))
        .route("/track/result", get(handlers::checkout::track_result))
        // ---------- Payment webhook ----------
        .route(
            "/webhook/midtrans",
            post(handlers::checkout::midtrans_webhook),
        )
        // ---------- Setup wizard ----------
        .route("/setup", get(handlers::setup::wizard))
        .route("/setup", post(handlers::setup::save_wizard))
        // ---------- Admin auth ----------
        .route(
            "/admin/login",
            get(handlers::admin::login_page).post(handlers::admin::login_submit),
        )
        .route("/admin/logout", post(handlers::admin::logout))
        // ---------- Admin dashboard ----------
        .route("/admin", get(handlers::admin::dashboard))
        .route("/admin/finance", get(handlers::admin::finance_report))
        // ---------- Admin products ----------
        .route("/admin/products", get(handlers::admin::products_list))
        .route("/admin/products/new", get(handlers::admin::product_form))
        .route(
            "/admin/products/:id/edit",
            get(handlers::admin::product_form),
        )
        .route("/admin/products/save", post(handlers::admin::product_save))
        .route(
            "/admin/products/:id/delete",
            post(handlers::admin::product_delete),
        )
        .route(
            "/admin/products/:id/toggle",
            post(handlers::admin::product_toggle),
        )
        // ---------- Admin categories ----------
        .route(
            "/admin/categories",
            get(handlers::admin::categories_page).post(handlers::admin::category_save),
        )
        .route(
            "/admin/categories/:id/delete",
            post(handlers::admin::category_delete),
        )
        // ---------- Admin orders ----------
        .route("/admin/orders", get(handlers::admin::orders_list))
        .route("/admin/orders/:id", get(handlers::admin::order_detail))
        .route(
            "/admin/orders/:id/update",
            post(handlers::admin::order_update),
        )
        // ---------- Admin banners ----------
        .route(
            "/admin/banners",
            get(handlers::admin::banners_page).post(handlers::admin::banner_save),
        )
        .route(
            "/admin/banners/:id/delete",
            post(handlers::admin::banner_delete),
        )
        // ---------- Admin coupons ----------
        .route(
            "/admin/coupons",
            get(handlers::admin::coupons_page).post(handlers::admin::coupon_save),
        )
        .route(
            "/admin/coupons/:id/delete",
            post(handlers::admin::coupon_delete),
        )
        // ---------- Admin pages ----------
        .route("/admin/pages", get(handlers::admin::pages_list))
        .route("/admin/pages/new", get(handlers::admin::page_form))
        .route("/admin/pages/:id/edit", get(handlers::admin::page_form))
        .route("/admin/pages/save", post(handlers::admin::page_save))
        .route(
            "/admin/pages/:id/delete",
            post(handlers::admin::page_delete),
        )
        // ---------- Admin appearance / themes ----------
        .route("/admin/appearance", get(handlers::admin::appearance_page))
        .route("/admin/appearance/theme", post(handlers::admin::set_theme))
        // ---------- Admin settings ----------
        .route(
            "/admin/settings",
            get(handlers::admin::settings_page).post(handlers::admin::settings_save),
        )
        .route("/admin/password", post(handlers::admin::password_change))
        .route(
            "/admin/payment",
            get(handlers::admin::payment_page).post(handlers::admin::payment_save),
        )
        // ---------- Admin uploads (AJAX) ----------
        .route("/admin/upload", post(handlers::admin::upload_image))
        // ---------- Static ----------
        .nest_service("/static", ServeDir::new("static"))
        .fallback(handlers::store::not_found)
        // Browser CSRF guard: SameSite keeps cookies off cross-site requests,
        // while Origin also rejects hostile sibling subdomains.
        .layer(axum::middleware::from_fn_with_state(
            csrf_origin,
            same_origin_guard,
        ))
        .layer(session_layer)
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::REFERRER_POLICY,
            axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::CONTENT_SECURITY_POLICY,
            axum::http::HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https: http:; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'; form-action 'self'",
            ),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("permissions-policy"),
            axum::http::HeaderValue::from_static(
                "camera=(), microphone=(), geolocation=(), payment=()",
            ),
        ))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::DefaultBodyLimit::max(12 * 1024 * 1024))
        .with_state(state);

    let app = if secure_cookie {
        app.layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
    } else {
        app
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("TokoKu berjalan di http://localhost:{port}  (admin: /admin)");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

async fn same_origin_guard(
    axum::extract::State(expected_origin): axum::extract::State<String>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::{Method, StatusCode};
    if matches!(
        *req.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    ) && req.uri().path() != "/webhook/midtrans"
    {
        let cross_site = req
            .headers()
            .get("sec-fetch-site")
            .and_then(|v| v.to_str().ok())
            == Some("cross-site");
        let wrong_origin = req
            .headers()
            .get(axum::http::header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|origin| origin.trim_end_matches('/') != expected_origin);
        if cross_site || wrong_origin {
            return (StatusCode::FORBIDDEN, "Cross-site request ditolak").into_response();
        }
    }
    next.run(req).await
}

fn origin_from_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if let Some(scheme_end) = trimmed.find("://") {
        let authority_end = trimmed[scheme_end + 3..]
            .find('/')
            .map_or(trimmed.len(), |i| scheme_end + 3 + i);
        trimmed[..authority_end].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Create an admin account on first boot. Never ship a predictable password.
fn ensure_default_admin(db: &db::Db) -> anyhow::Result<()> {
    if db::count_users(db)? == 0 {
        let user = std::env::var("ADMIN_USER").unwrap_or_else(|_| "admin".into());
        let pass = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
            use rand::{distributions::Alphanumeric, Rng};
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(24)
                .map(char::from)
                .collect()
        });
        let hash = services::auth::hash_password(&pass)?;
        db::create_user(db, &user, "", &hash)?;
        tracing::warn!(
            "Admin dibuat -> user: '{user}', password awal: '{pass}' (simpan dan segera ganti)"
        );
    }
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("Shutting down gracefully...");
}
