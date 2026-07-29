// Storefront (public) handlers: home, product listing, product detail, category,
// search, content pages, and review submission.
use crate::filters;
use crate::db::{self, ProductFilter};
use crate::models::{Banner, Category, Page, Product, Review, Settings};
use crate::services::cart;
use crate::services::themes::Theme;
use crate::state::{AppState, StoreContext};
use askama::Template;
use super::TemplateResponse;
use axum::extract::{Form, Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use tower_sessions::Session;

use super::server_error;

// Common fields shared by all storefront templates, embedded as `s`.
pub struct Shared {
    pub settings: Settings,
    pub theme: &'static Theme,
    pub theme_css: String,
    pub nav_categories: Vec<Category>,
    pub footer_pages: Vec<Page>,
    pub cart_count: i64,
    pub current_year: i64,
}

impl Shared {
    pub async fn build(state: &AppState, session: &Session) -> Self {
        let cart_items = cart::get_cart(session).await;
        let ctx = StoreContext::build(state, cart::cart_count(&cart_items));
        Shared {
            settings: ctx.settings,
            theme: ctx.theme,
            theme_css: ctx.theme_css,
            nav_categories: ctx.categories,
            footer_pages: ctx.footer_pages,
            cart_count: ctx.cart_count,
            current_year: 2026,
        }
    }
    // Template helpers
    pub fn store_name(&self) -> &str { &self.settings.store_name }
    pub fn currency(&self) -> &str { &self.settings.currency }
    pub fn wa(&self) -> &str { &self.settings.whatsapp }
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "store/home.html")]
struct HomeTemplate {
    s: Shared,
    banners: Vec<Banner>,
    featured: Vec<Product>,
    newest: Vec<Product>,
    categories: Vec<Category>,
    best_sellers: Vec<Product>,
}

#[derive(Template)]
#[template(path = "store/products.html")]
struct ProductListTemplate {
    s: Shared,
    products: Vec<Product>,
    categories: Vec<Category>,
    title: String,
    active_category: String,
    sort: String,
    search: String,
    total: i64,
    page: i64,
    total_pages: i64,
}

#[derive(Template)]
#[template(path = "store/product_detail.html")]
struct ProductDetailTemplate {
    s: Shared,
    product: Product,
    images: Vec<String>,
    reviews: Vec<Review>,
    related: Vec<Product>,
    avg_rating: f64,
    review_count: i64,
}

#[derive(Template)]
#[template(path = "store/page.html")]
struct PageTemplate {
    s: Shared,
    page: Page,
}

#[derive(Template)]
#[template(path = "store/not_found.html")]
struct NotFoundTemplate {
    s: Shared,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn home(State(state): State<AppState>, session: Session) -> Response {
    let s = Shared::build(&state, &session).await;

    // If the store has never been configured, guide the owner to setup.
    if !s.settings.setup_done && db::count_products(&state.db, &ProductFilter { include_inactive: true, ..Default::default() }).unwrap_or(0) == 0 {
        return Redirect::to("/setup").into_response();
    }

    let banners = db::list_banners(&state.db, true).unwrap_or_default();
    let featured = db::list_products(&state.db, &ProductFilter { featured_only: true, limit: 8, sort: "newest".into(), ..Default::default() }).unwrap_or_default();
    let newest = db::list_products(&state.db, &ProductFilter { limit: 8, sort: "newest".into(), ..Default::default() }).unwrap_or_default();
    let best_sellers = db::list_products(&state.db, &ProductFilter { limit: 8, sort: "popular".into(), ..Default::default() }).unwrap_or_default();
    let categories = db::list_categories(&state.db).unwrap_or_default();

    HomeTemplate { s, banners, featured, newest, categories, best_sellers }.page()
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub q: Option<String>,
}

pub async fn product_list(State(state): State<AppState>, session: Session, Query(q): Query<ListQuery>) -> Response {
    render_product_list(&state, &session, None, q, "Semua Produk").await
}

pub async fn category_page(State(state): State<AppState>, session: Session, Path(slug): Path<String>, Query(q): Query<ListQuery>) -> Response {
    match db::get_category_by_slug(&state.db, &slug) {
        Ok(Some(cat)) => {
            let title = cat.name.clone();
            render_product_list(&state, &session, Some(cat), q, &title).await
        }
        Ok(None) => not_found(State(state), session).await,
        Err(e) => server_error(e),
    }
}

pub async fn search(State(state): State<AppState>, session: Session, Query(q): Query<ListQuery>) -> Response {
    let term = q.q.clone().unwrap_or_default();
    let title = format!("Hasil pencarian: \"{}\"", term);
    render_product_list(&state, &session, None, q, &title).await
}

async fn render_product_list(state: &AppState, session: &Session, category: Option<Category>, q: ListQuery, title: &str) -> Response {
    let s = Shared::build(state, session).await;
    let sort = q.sort.unwrap_or_else(|| "newest".into());
    let page = q.page.unwrap_or(1).max(1);
    let per_page = 12;
    let search = q.q.clone().unwrap_or_default();

    let filter = ProductFilter {
        category_id: category.as_ref().map(|c| c.id),
        search: if search.is_empty() { None } else { Some(search.clone()) },
        sort: sort.clone(),
        limit: per_page,
        offset: (page - 1) * per_page,
        ..Default::default()
    };
    let count_filter = ProductFilter {
        category_id: category.as_ref().map(|c| c.id),
        search: if search.is_empty() { None } else { Some(search.clone()) },
        ..Default::default()
    };

    let products = match db::list_products(&state.db, &filter) {
        Ok(p) => p,
        Err(e) => return server_error(e),
    };
    let total = db::count_products(&state.db, &count_filter).unwrap_or(0);
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i64;
    let categories = db::list_categories(&state.db).unwrap_or_default();
    let active_category = category.as_ref().map(|c| c.slug.clone()).unwrap_or_default();

    ProductListTemplate {
        s,
        products,
        categories,
        title: title.to_string(),
        active_category,
        sort,
        search,
        total,
        page,
        total_pages,
    }
    .page()
}

pub async fn product_detail(State(state): State<AppState>, session: Session, Path(slug): Path<String>) -> Response {
    let s = Shared::build(&state, &session).await;
    match db::get_product_by_slug(&state.db, &slug) {
        Ok(Some(product)) => {
            db::increment_views(&state.db, product.id);
            let images = product.image_list();
            let reviews = db::list_reviews_for_product(&state.db, product.id).unwrap_or_default();
            let related = db::related_products(&state.db, &product, 4).unwrap_or_default();
            let avg_rating = product.avg_rating;
            let review_count = product.review_count;
            ProductDetailTemplate { s, product, images, reviews, related, avg_rating, review_count }.page()
        }
        Ok(None) => not_found(State(state), session).await,
        Err(e) => server_error(e),
    }
}

pub async fn content_page(State(state): State<AppState>, session: Session, Path(slug): Path<String>) -> Response {
    let s = Shared::build(&state, &session).await;
    match db::get_page_by_slug(&state.db, &slug) {
        Ok(Some(page)) if page.is_published => PageTemplate { s, page }.page(),
        Ok(_) => not_found(State(state), session).await,
        Err(e) => server_error(e),
    }
}

#[derive(Deserialize)]
pub struct ReviewForm {
    pub customer_name: String,
    pub rating: i64,
    pub comment: String,
}

pub async fn submit_review(State(state): State<AppState>, Path(slug): Path<String>, Form(form): Form<ReviewForm>) -> Response {
    if let Ok(Some(product)) = db::get_product_by_slug(&state.db, &slug) {
        let rating = form.rating.clamp(1, 5);
        let name = if form.customer_name.trim().is_empty() { "Anonim".to_string() } else { form.customer_name };
        let _ = db::create_review(&state.db, product.id, &name, rating, &form.comment);
    }
    Redirect::to(&format!("/product/{}#reviews", slug)).into_response()
}

pub async fn not_found(State(state): State<AppState>, session: Session) -> Response {
    let s = Shared::build(&state, &session).await;
    let mut resp = NotFoundTemplate { s }.page();
    *resp.status_mut() = axum::http::StatusCode::NOT_FOUND;
    resp
}
