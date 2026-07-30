// Admin panel handlers. Provides a no-code management UI: dashboard, products,
// categories, orders, banners, coupons, pages, appearance (themes), settings,
// payment configuration, and image uploads.
use crate::db::{self, DashboardStats, ProductFilter};
use crate::filters;
use crate::models::*;
use crate::services::themes::Theme;
use crate::services::{auth, helpers, themes};
use crate::state::AppState;
use askama::Template;
use axum::extract::{ConnectInfo, Form, Multipart, Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tower_sessions::Session;

use super::TemplateResponse;
use super::{redirect, server_error};

const SESSION_USER_KEY: &str = "admin_user";

// ---------------------------------------------------------------------------
// Auth guard
// ---------------------------------------------------------------------------

/// Returns Some(username) when logged in, else None.
async fn current_user(session: &Session) -> Option<String> {
    session.get::<String>(SESSION_USER_KEY).await.ok().flatten()
}

/// Guard used by protected handlers. On failure returns a redirect response.
macro_rules! require_login {
    ($session:expr) => {
        match current_user(&$session).await {
            Some(u) => u,
            None => return Redirect::to("/admin/login").into_response(),
        }
    };
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "admin/login.html")]
struct LoginTemplate {
    error: String,
    store_name: String,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct DashboardTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    stats: DashboardStats,
    currency: String,
    max_sales: i64,
}

#[derive(Template)]
#[template(path = "admin/products.html")]
struct ProductsTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    products: Vec<Product>,
    categories: Vec<Category>,
    currency: String,
    search: String,
    total: i64,
}

#[derive(Template)]
#[template(path = "admin/product_form.html")]
struct ProductFormTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    product: Product,
    categories: Vec<Category>,
    images: Vec<String>,
    is_edit: bool,
    currency: String,
}

#[derive(Template)]
#[template(path = "admin/categories.html")]
struct CategoriesTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    categories: Vec<Category>,
}

#[derive(Template)]
#[template(path = "admin/orders.html")]
struct OrdersTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    orders: Vec<Order>,
    currency: String,
    filter: String,
    counts: OrderCounts,
}

struct OrderCounts {
    all: i64,
    new: i64,
    processing: i64,
    shipped: i64,
    completed: i64,
    cancelled: i64,
}

#[derive(Template)]
#[template(path = "admin/order_detail.html")]
struct OrderDetailTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    order: Order,
    currency: String,
}

#[derive(Template)]
#[template(path = "admin/banners.html")]
struct BannersTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    banners: Vec<Banner>,
}

#[derive(Template)]
#[template(path = "admin/coupons.html")]
struct CouponsTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    coupons: Vec<Coupon>,
    currency: String,
}

#[derive(Template)]
#[template(path = "admin/pages.html")]
struct PagesTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    pages: Vec<Page>,
}

#[derive(Template)]
#[template(path = "admin/page_form.html")]
struct PageFormTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    page: Page,
    is_edit: bool,
}

#[derive(Template)]
#[template(path = "admin/appearance.html")]
struct AppearanceTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    themes: &'static [Theme],
    current_theme: String,
}

#[derive(Template)]
#[template(path = "admin/settings.html")]
struct SettingsTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    settings: Settings,
}

#[derive(Template)]
#[template(path = "admin/payment.html")]
struct PaymentTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    settings: Settings,
    payment: PaymentConfig,
}

#[derive(Template)]
#[template(path = "admin/finance.html")]
struct FinanceTemplate {
    user: String,
    store_name: String,
    active: &'static str,
    currency: String,
    report: db::FinanceReport,
    days: i64,
}

fn store_name(state: &AppState) -> String {
    state.settings().store_name
}

fn currency(state: &AppState) -> String {
    state.settings().currency
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

pub async fn login_page(State(state): State<AppState>, session: Session) -> Response {
    if current_user(&session).await.is_some() {
        return Redirect::to("/admin").into_response();
    }
    LoginTemplate {
        error: String::new(),
        store_name: store_name(&state),
    }
    .page()
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_submit(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Response {
    // Include the username to prevent one client's attempts against unrelated
    // accounts from sharing a bucket. Do not trust spoofable forwarding headers.
    let limit_key = format!("{}:{}", addr.ip(), form.username.trim().to_lowercase());
    if state.login_limiter.is_blocked(&limit_key) {
        return (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            LoginTemplate {
                error: "Terlalu banyak percobaan login. Coba lagi dalam 15 menit.".into(),
                store_name: store_name(&state),
            }
            .render()
            .unwrap_or_else(|_| "Terlalu banyak percobaan login.".into()),
        )
            .into_response();
    }
    match db::get_user_by_username(&state.db, form.username.trim()) {
        Ok(Some((_, username, hash))) if auth::verify_password(&form.password, &hash) => {
            state.login_limiter.success(&limit_key);
            let _ = session.insert(SESSION_USER_KEY, username).await;
            Redirect::to("/admin").into_response()
        }
        _ => {
            state.login_limiter.failure(limit_key);
            LoginTemplate {
                error: "Username atau password salah.".into(),
                store_name: store_name(&state),
            }
            .page()
        }
    }
}

pub async fn logout(session: Session) -> Response {
    let _ = session.remove::<String>(SESSION_USER_KEY).await;
    Redirect::to("/admin/login").into_response()
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

pub async fn dashboard(State(state): State<AppState>, session: Session) -> Response {
    let user = require_login!(session);
    let stats = match db::dashboard_stats(&state.db) {
        Ok(s) => s,
        Err(e) => return server_error(e),
    };
    let max_sales = stats
        .sales_last_7_days
        .iter()
        .map(|(_, v)| *v)
        .max()
        .unwrap_or(0)
        .max(1);
    DashboardTemplate {
        user,
        store_name: store_name(&state),
        active: "dashboard",
        stats,
        currency: currency(&state),
        max_sales,
    }
    .into_response()
}

// ---------------------------------------------------------------------------
// Finance report (Laporan Keuangan)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct FinanceQuery {
    pub period: Option<i64>,
}

pub async fn finance_report(
    State(state): State<AppState>,
    session: Session,
    Query(q): Query<FinanceQuery>,
) -> Response {
    let user = require_login!(session);
    // Restrict to a small set of sensible windows; default to 30 days.
    let days = match q.period.unwrap_or(30) {
        7 => 7,
        90 => 90,
        365 => 365,
        _ => 30,
    };
    let report = match db::finance_report(&state.db, days) {
        Ok(r) => r,
        Err(e) => return server_error(e),
    };
    FinanceTemplate {
        user,
        store_name: store_name(&state),
        active: "finance",
        currency: currency(&state),
        report,
        days,
    }
    .into_response()
}

// ---------------------------------------------------------------------------
// Products
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ProductQuery {
    pub q: Option<String>,
}

pub async fn products_list(
    State(state): State<AppState>,
    session: Session,
    Query(q): Query<ProductQuery>,
) -> Response {
    let user = require_login!(session);
    let search = q.q.unwrap_or_default();
    let filter = ProductFilter {
        search: if search.is_empty() {
            None
        } else {
            Some(search.clone())
        },
        include_inactive: true,
        sort: "newest".into(),
        limit: 200,
        ..Default::default()
    };
    let products = db::list_products(&state.db, &filter).unwrap_or_default();
    let total = db::count_products(
        &state.db,
        &ProductFilter {
            include_inactive: true,
            ..Default::default()
        },
    )
    .unwrap_or(0);
    let categories = db::list_categories(&state.db).unwrap_or_default();
    ProductsTemplate {
        user,
        store_name: store_name(&state),
        active: "products",
        products,
        categories,
        currency: currency(&state),
        search,
        total,
    }
    .into_response()
}

pub async fn product_form(
    State(state): State<AppState>,
    session: Session,
    id: Option<Path<i64>>,
) -> Response {
    let user = require_login!(session);
    let categories = db::list_categories(&state.db).unwrap_or_default();
    let (product, images, is_edit) = match id {
        Some(Path(pid)) => match db::get_product(&state.db, pid) {
            Ok(Some(p)) => {
                let imgs = p.image_list();
                let imgs = if imgs == vec!["/static/img/placeholder.svg".to_string()] {
                    vec![]
                } else {
                    imgs
                };
                (p, imgs, true)
            }
            _ => return redirect("/admin/products"),
        },
        None => (
            Product {
                is_active: true,
                track_stock: true,
                ..Default::default()
            },
            vec![],
            false,
        ),
    };
    ProductFormTemplate {
        user,
        store_name: store_name(&state),
        active: "products",
        product,
        categories,
        images,
        is_edit,
        currency: currency(&state),
    }
    .into_response()
}

#[derive(Deserialize)]
pub struct ProductSaveForm {
    #[serde(default)]
    pub id: Option<i64>,
    pub name: String,
    #[serde(default)]
    pub sku: String,
    #[serde(default)]
    pub short_desc: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub price: i64,
    #[serde(default)]
    pub compare_price: i64,
    #[serde(default)]
    pub cost_price: i64,
    #[serde(default)]
    pub stock: i64,
    #[serde(default)]
    pub weight_grams: i64,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default)]
    pub images: String, // comma or newline separated URLs
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub is_active: Option<String>,
    #[serde(default)]
    pub is_featured: Option<String>,
    #[serde(default)]
    pub track_stock: Option<String>,
}

fn cb(v: &Option<String>) -> bool {
    matches!(v.as_deref(), Some("on") | Some("true") | Some("1"))
}

fn parse_images(raw: &str) -> String {
    let list: Vec<String> = raw
        .split(['\n', ','])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    serde_json::to_string(&list).unwrap_or_else(|_| "[]".into())
}

pub async fn product_save(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<ProductSaveForm>,
) -> Response {
    let _user = require_login!(session);
    if form.name.trim().is_empty() {
        return redirect("/admin/products/new");
    }
    let category_id = form.category_id.filter(|&c| c > 0);
    // Generate slug; keep existing slug on edit to avoid breaking links.
    let slug = match form.id {
        Some(pid) => db::get_product(&state.db, pid)
            .ok()
            .flatten()
            .map(|p| p.slug)
            .unwrap_or_else(|| helpers::make_slug(&form.name)),
        None => helpers::make_slug(&form.name),
    };
    let product = Product {
        name: form.name.trim().to_string(),
        slug,
        sku: form.sku.trim().to_string(),
        short_desc: form.short_desc.trim().to_string(),
        description: form.description.trim().to_string(),
        price: form.price.max(0),
        compare_price: form.compare_price.max(0),
        cost_price: form.cost_price.max(0),
        stock: form.stock.max(0),
        weight_grams: form.weight_grams.max(0),
        category_id,
        images: parse_images(&form.images),
        tags: form.tags.trim().to_string(),
        is_active: cb(&form.is_active),
        is_featured: cb(&form.is_featured),
        track_stock: cb(&form.track_stock),
        ..Default::default()
    };
    match db::upsert_product(&state.db, &product, form.id) {
        Ok(_) => redirect("/admin/products"),
        Err(e) => server_error(e),
    }
}

pub async fn product_delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    let _user = require_login!(session);
    let _ = db::delete_product(&state.db, id);
    redirect("/admin/products")
}

pub async fn product_toggle(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    let _user = require_login!(session);
    let _ = db::toggle_product_active(&state.db, id);
    redirect("/admin/products")
}

// ---------------------------------------------------------------------------
// Categories
// ---------------------------------------------------------------------------

pub async fn categories_page(State(state): State<AppState>, session: Session) -> Response {
    let user = require_login!(session);
    let categories = db::list_categories(&state.db).unwrap_or_default();
    CategoriesTemplate {
        user,
        store_name: store_name(&state),
        active: "categories",
        categories,
    }
    .page()
}

#[derive(Deserialize)]
pub struct CategorySaveForm {
    #[serde(default)]
    pub id: Option<i64>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub image_url: String,
}

pub async fn category_save(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<CategorySaveForm>,
) -> Response {
    let _user = require_login!(session);
    if form.name.trim().is_empty() {
        return redirect("/admin/categories");
    }
    let slug = helpers::make_slug(&form.name);
    let res = match form.id.filter(|&i| i > 0) {
        Some(id) => db::update_category(
            &state.db,
            id,
            form.name.trim(),
            &slug,
            form.description.trim(),
            form.icon.trim(),
            form.image_url.trim(),
        ),
        None => db::create_category(
            &state.db,
            form.name.trim(),
            &slug,
            form.description.trim(),
            form.icon.trim(),
            form.image_url.trim(),
        )
        .map(|_| ()),
    };
    match res {
        Ok(_) => redirect("/admin/categories"),
        Err(e) => server_error(e),
    }
}

pub async fn category_delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    let _user = require_login!(session);
    let _ = db::delete_category(&state.db, id);
    redirect("/admin/categories")
}

// ---------------------------------------------------------------------------
// Orders
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct OrderQuery {
    pub status: Option<String>,
}

pub async fn orders_list(
    State(state): State<AppState>,
    session: Session,
    Query(q): Query<OrderQuery>,
) -> Response {
    let user = require_login!(session);
    let filter = q.status.unwrap_or_else(|| "all".into());
    let status = if filter == "all" {
        None
    } else {
        Some(filter.as_str())
    };
    let orders = db::list_orders(&state.db, status, 200, 0).unwrap_or_default();
    let counts = OrderCounts {
        all: db::count_orders(&state.db, None).unwrap_or(0),
        new: db::count_orders(&state.db, Some("new")).unwrap_or(0),
        processing: db::count_orders(&state.db, Some("processing")).unwrap_or(0),
        shipped: db::count_orders(&state.db, Some("shipped")).unwrap_or(0),
        completed: db::count_orders(&state.db, Some("completed")).unwrap_or(0),
        cancelled: db::count_orders(&state.db, Some("cancelled")).unwrap_or(0),
    };
    OrdersTemplate {
        user,
        store_name: store_name(&state),
        active: "orders",
        orders,
        currency: currency(&state),
        filter,
        counts,
    }
    .into_response()
}

pub async fn order_detail(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    let user = require_login!(session);
    match db::get_order(&state.db, id) {
        Ok(Some(order)) => OrderDetailTemplate {
            user,
            store_name: store_name(&state),
            active: "orders",
            order,
            currency: currency(&state),
        }
        .page(),
        _ => redirect("/admin/orders"),
    }
}

#[derive(Deserialize)]
pub struct OrderUpdateForm {
    pub order_status: String,
    pub payment_status: String,
    #[serde(default)]
    pub tracking_number: String,
}

pub async fn order_update(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
    Form(form): Form<OrderUpdateForm>,
) -> Response {
    let _user = require_login!(session);
    const ORDER_STATUSES: &[&str] = &["new", "processing", "shipped", "completed", "cancelled"];
    const PAYMENT_STATUSES: &[&str] = &["pending", "paid", "failed", "refunded"];
    if !ORDER_STATUSES.contains(&form.order_status.as_str())
        || !PAYMENT_STATUSES.contains(&form.payment_status.as_str())
    {
        return (axum::http::StatusCode::BAD_REQUEST, "Status tidak valid").into_response();
    }
    if let Err(e) = db::update_order_status(
        &state.db,
        id,
        &form.order_status,
        &form.payment_status,
        form.tracking_number.trim(),
    ) {
        return server_error(e);
    }
    redirect(&format!("/admin/orders/{}", id))
}

// ---------------------------------------------------------------------------
// Banners
// ---------------------------------------------------------------------------

pub async fn banners_page(State(state): State<AppState>, session: Session) -> Response {
    let user = require_login!(session);
    let banners = db::list_banners(&state.db, false).unwrap_or_default();
    BannersTemplate {
        user,
        store_name: store_name(&state),
        active: "banners",
        banners,
    }
    .page()
}

#[derive(Deserialize)]
pub struct BannerSaveForm {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    pub image_url: String,
    #[serde(default)]
    pub link_url: String,
    #[serde(default)]
    pub button_text: String,
    #[serde(default)]
    pub sort_order: i64,
}

pub async fn banner_save(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<BannerSaveForm>,
) -> Response {
    let _user = require_login!(session);
    let banner = Banner {
        title: form.title.trim().to_string(),
        subtitle: form.subtitle.trim().to_string(),
        image_url: form.image_url.trim().to_string(),
        link_url: form.link_url.trim().to_string(),
        button_text: if form.button_text.trim().is_empty() {
            "Belanja Sekarang".into()
        } else {
            form.button_text.trim().to_string()
        },
        sort_order: form.sort_order,
        is_active: true,
        ..Default::default()
    };
    let res = match form.id.filter(|&i| i > 0) {
        Some(id) => db::update_banner(&state.db, id, &banner),
        None => db::create_banner(&state.db, &banner).map(|_| ()),
    };
    match res {
        Ok(_) => redirect("/admin/banners"),
        Err(e) => server_error(e),
    }
}

pub async fn banner_delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    let _user = require_login!(session);
    let _ = db::delete_banner(&state.db, id);
    redirect("/admin/banners")
}

// ---------------------------------------------------------------------------
// Coupons
// ---------------------------------------------------------------------------

pub async fn coupons_page(State(state): State<AppState>, session: Session) -> Response {
    let user = require_login!(session);
    let coupons = db::list_coupons(&state.db).unwrap_or_default();
    CouponsTemplate {
        user,
        store_name: store_name(&state),
        active: "coupons",
        coupons,
        currency: currency(&state),
    }
    .page()
}

#[derive(Deserialize)]
pub struct CouponSaveForm {
    #[serde(default)]
    pub id: Option<i64>,
    pub code: String,
    pub r#type: String,
    pub value: i64,
    #[serde(default)]
    pub min_purchase: i64,
    #[serde(default)]
    pub max_uses: i64,
}

pub async fn coupon_save(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<CouponSaveForm>,
) -> Response {
    let _user = require_login!(session);
    if form.code.trim().is_empty() {
        return redirect("/admin/coupons");
    }
    let coupon = Coupon {
        code: form.code.trim().to_uppercase(),
        r#type: if form.r#type == "fixed" {
            "fixed".into()
        } else {
            "percent".into()
        },
        value: if form.r#type == "percent" {
            form.value.clamp(0, 100)
        } else {
            form.value.max(0)
        },
        min_purchase: form.min_purchase.max(0),
        max_uses: form.max_uses.max(0),
        is_active: true,
        ..Default::default()
    };
    let res = match form.id.filter(|&i| i > 0) {
        Some(id) => db::update_coupon(&state.db, id, &coupon),
        None => db::create_coupon(&state.db, &coupon).map(|_| ()),
    };
    match res {
        Ok(_) => redirect("/admin/coupons"),
        Err(e) => server_error(e),
    }
}

pub async fn coupon_delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    let _user = require_login!(session);
    let _ = db::delete_coupon(&state.db, id);
    redirect("/admin/coupons")
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

pub async fn pages_list(State(state): State<AppState>, session: Session) -> Response {
    let user = require_login!(session);
    let pages = db::list_pages(&state.db, false).unwrap_or_default();
    PagesTemplate {
        user,
        store_name: store_name(&state),
        active: "pages",
        pages,
    }
    .page()
}

pub async fn page_form(
    State(state): State<AppState>,
    session: Session,
    id: Option<Path<i64>>,
) -> Response {
    let user = require_login!(session);
    let (page, is_edit) = match id {
        Some(Path(pid)) => match db::list_pages(&state.db, false)
            .unwrap_or_default()
            .into_iter()
            .find(|p| p.id == pid)
        {
            Some(p) => (p, true),
            None => return redirect("/admin/pages"),
        },
        None => (
            Page {
                is_published: true,
                show_in_footer: true,
                ..Default::default()
            },
            false,
        ),
    };
    PageFormTemplate {
        user,
        store_name: store_name(&state),
        active: "pages",
        page,
        is_edit,
    }
    .page()
}

#[derive(Deserialize)]
pub struct PageSaveForm {
    #[serde(default)]
    pub id: Option<i64>,
    pub title: String,
    #[serde(default)]
    pub content: String,
}

pub async fn page_save(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<PageSaveForm>,
) -> Response {
    let _user = require_login!(session);
    if form.title.trim().is_empty() {
        return redirect("/admin/pages");
    }
    let slug = match form.id.filter(|&i| i > 0) {
        Some(pid) => db::list_pages(&state.db, false)
            .unwrap_or_default()
            .into_iter()
            .find(|p| p.id == pid)
            .map(|p| p.slug)
            .unwrap_or_else(|| helpers::make_slug(&form.title)),
        None => helpers::make_slug(&form.title),
    };
    let _ = db::upsert_page(
        &state.db,
        form.title.trim(),
        &slug,
        form.content.trim(),
        form.id.filter(|&i| i > 0),
    );
    redirect("/admin/pages")
}

pub async fn page_delete(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Response {
    let _user = require_login!(session);
    let _ = db::delete_page(&state.db, id);
    redirect("/admin/pages")
}

// ---------------------------------------------------------------------------
// Appearance / themes
// ---------------------------------------------------------------------------

pub async fn appearance_page(State(state): State<AppState>, session: Session) -> Response {
    let user = require_login!(session);
    let current_theme = state.settings().theme;
    AppearanceTemplate {
        user,
        store_name: store_name(&state),
        active: "appearance",
        themes: themes::THEMES,
        current_theme,
    }
    .into_response()
}

#[derive(Deserialize)]
pub struct ThemeForm {
    pub theme: String,
}

pub async fn set_theme(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<ThemeForm>,
) -> Response {
    let _user = require_login!(session);
    // Validate theme id exists before saving.
    if themes::THEMES.iter().any(|t| t.id == form.theme) {
        let _ = db::set_theme(&state.db, &form.theme);
    }
    redirect("/admin/appearance")
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub async fn settings_page(State(state): State<AppState>, session: Session) -> Response {
    let user = require_login!(session);
    let settings = state.settings();
    SettingsTemplate {
        user,
        store_name: store_name(&state),
        active: "settings",
        settings,
    }
    .page()
}

#[derive(Deserialize)]
pub struct SettingsSaveForm {
    pub store_name: String,
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub logo_url: String,
    #[serde(default)]
    pub hero_image_url: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub whatsapp: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub instagram: String,
    #[serde(default)]
    pub facebook: String,
    #[serde(default)]
    pub tiktok: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub shipping_flat: Option<i64>,
    #[serde(default)]
    pub shipping_free_min: Option<i64>,
    #[serde(default)]
    pub tax_percent: Option<f64>,
    #[serde(default)]
    pub meta_keywords: String,
    #[serde(default)]
    pub announcement: String,
    #[serde(default)]
    pub announcement_on: Option<String>,
}

pub async fn settings_save(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<SettingsSaveForm>,
) -> Response {
    let _user = require_login!(session);
    let mut settings = state.settings();
    settings.store_name = if form.store_name.trim().is_empty() {
        "Toko Saya".into()
    } else {
        form.store_name.trim().to_string()
    };
    settings.tagline = form.tagline.trim().to_string();
    settings.description = form.description.trim().to_string();
    settings.logo_url = form.logo_url.trim().to_string();
    settings.hero_image_url = form.hero_image_url.trim().to_string();
    settings.phone = form.phone.trim().to_string();
    settings.whatsapp = form.whatsapp.trim().to_string();
    settings.email = form.email.trim().to_string();
    settings.address = form.address.trim().to_string();
    settings.instagram = form.instagram.trim().to_string();
    settings.facebook = form.facebook.trim().to_string();
    settings.tiktok = form.tiktok.trim().to_string();
    settings.currency = if form.currency.trim().is_empty() {
        "Rp".into()
    } else {
        form.currency.trim().to_string()
    };
    settings.shipping_flat = form.shipping_flat.unwrap_or(settings.shipping_flat).max(0);
    settings.shipping_free_min = form
        .shipping_free_min
        .unwrap_or(settings.shipping_free_min)
        .max(0);
    settings.tax_percent = form
        .tax_percent
        .unwrap_or(settings.tax_percent)
        .clamp(0.0, 100.0);
    settings.meta_keywords = form.meta_keywords.trim().to_string();
    settings.announcement = form.announcement.trim().to_string();
    settings.announcement_on = cb(&form.announcement_on);
    settings.setup_done = true;
    match db::update_settings(&state.db, &settings) {
        Ok(_) => redirect("/admin/settings?saved=1"),
        Err(e) => server_error(e),
    }
}

#[derive(Deserialize)]
pub struct PasswordChangeForm {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

pub async fn password_change(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<PasswordChangeForm>,
) -> Response {
    let user = require_login!(session);
    if form.new_password.len() < 10 || form.new_password != form.confirm_password {
        return redirect("/admin/settings?password=invalid");
    }
    let valid = matches!(
        db::get_user_by_username(&state.db, &user),
        Ok(Some((_, _, ref hash))) if auth::verify_password(&form.current_password, hash)
    );
    if !valid {
        return redirect("/admin/settings?password=wrong");
    }
    let hash = match auth::hash_password(&form.new_password) {
        Ok(h) => h,
        Err(e) => return server_error(e),
    };
    match db::update_user_password(&state.db, &user, &hash) {
        Ok(_) => redirect("/admin/settings?password=saved"),
        Err(e) => server_error(e),
    }
}

// ---------------------------------------------------------------------------
// Payment configuration
// ---------------------------------------------------------------------------

pub async fn payment_page(State(state): State<AppState>, session: Session) -> Response {
    let user = require_login!(session);
    let settings = state.settings();
    let payment = settings.payment();
    PaymentTemplate {
        user,
        store_name: store_name(&state),
        active: "payment",
        settings,
        payment,
    }
    .page()
}

#[derive(Deserialize)]
pub struct PaymentSaveForm {
    #[serde(default)]
    pub bank_transfer_enabled: Option<String>,
    #[serde(default)]
    pub bank_name: String,
    #[serde(default)]
    pub bank_account_number: String,
    #[serde(default)]
    pub bank_account_holder: String,
    #[serde(default)]
    pub qris_enabled: Option<String>,
    #[serde(default)]
    pub qris_image_url: String,
    #[serde(default)]
    pub cod_enabled: Option<String>,
    #[serde(default)]
    pub ewallet_enabled: Option<String>,
    #[serde(default)]
    pub ewallet_name: String,
    #[serde(default)]
    pub ewallet_number: String,
    #[serde(default)]
    pub midtrans_enabled: Option<String>,
    #[serde(default)]
    pub midtrans_server_key: String,
    #[serde(default)]
    pub midtrans_client_key: String,
    #[serde(default)]
    pub midtrans_production: Option<String>,
}

pub async fn payment_save(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<PaymentSaveForm>,
) -> Response {
    let _user = require_login!(session);
    let mut settings = state.settings();
    let payment = json!({
        "bank_transfer_enabled": cb(&form.bank_transfer_enabled),
        "bank_name": form.bank_name.trim(),
        "bank_account_number": form.bank_account_number.trim(),
        "bank_account_holder": form.bank_account_holder.trim(),
        "qris_enabled": cb(&form.qris_enabled),
        "qris_image_url": form.qris_image_url.trim(),
        "cod_enabled": cb(&form.cod_enabled),
        "ewallet_enabled": cb(&form.ewallet_enabled),
        "ewallet_name": form.ewallet_name.trim(),
        "ewallet_number": form.ewallet_number.trim(),
        "midtrans_enabled": cb(&form.midtrans_enabled),
        "midtrans_server_key": form.midtrans_server_key.trim(),
        "midtrans_client_key": form.midtrans_client_key.trim(),
        "midtrans_production": cb(&form.midtrans_production)
    });
    settings.payment_config = payment.to_string();
    match db::update_settings(&state.db, &settings) {
        Ok(_) => redirect("/admin/payment?saved=1"),
        Err(e) => server_error(e),
    }
}

// ---------------------------------------------------------------------------
// Image upload (AJAX, returns JSON {url})
// ---------------------------------------------------------------------------

pub async fn upload_image(
    State(_state): State<AppState>,
    session: Session,
    mut multipart: Multipart,
) -> Response {
    if current_user(&session).await.is_none() {
        return (axum::http::StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    while let Ok(Some(field)) = multipart.next_field().await {
        let filename = field.file_name().unwrap_or("image.jpg").to_string();
        if let Ok(bytes) = field.bytes().await {
            match crate::services::upload::save_image(&bytes, &filename) {
                Ok(url) => return Json(json!({ "success": true, "url": url })).into_response(),
                Err(e) => {
                    return Json(json!({ "success": false, "error": e.to_string() }))
                        .into_response()
                }
            }
        }
    }
    Json(json!({ "success": false, "error": "no file" })).into_response()
}
