// Cart handlers: view, add, update, remove, and a small JSON count endpoint used
// by the header badge.
use crate::db;
use crate::filters;
use crate::models::CartItem;
use crate::services::cart;
use crate::state::AppState;
use askama::Template;
use axum::extract::{Form, State};
use axum::response::{IntoResponse, Json, Redirect, Response};
use serde::Deserialize;
use serde_json::json;
use tower_sessions::Session;

use super::store::Shared;
use super::TemplateResponse;

#[derive(Template)]
#[template(path = "store/cart.html")]
struct CartTemplate {
    s: Shared,
    items: Vec<CartItem>,
    subtotal: i64,
    shipping_estimate: i64,
    free_shipping_min: i64,
    qualifies_free: bool,
}

pub async fn view_cart(State(state): State<AppState>, session: Session) -> Response {
    let s = Shared::build(&state, &session).await;
    let items = cart::get_cart(&session).await;
    let subtotal = cart::cart_subtotal(&items);
    let settings = &s.settings;
    let qualifies_free = settings.shipping_free_min > 0 && subtotal >= settings.shipping_free_min;
    let shipping_estimate = if qualifies_free {
        0
    } else {
        settings.shipping_flat
    };
    let free_shipping_min = settings.shipping_free_min;
    CartTemplate {
        s,
        items,
        subtotal,
        shipping_estimate,
        free_shipping_min,
        qualifies_free,
    }
    .page()
}

#[derive(Deserialize)]
pub struct AddForm {
    pub product_id: i64,
    pub quantity: Option<i64>,
    #[serde(default)]
    pub redirect: Option<String>,
}

pub async fn add_to_cart(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<AddForm>,
) -> Response {
    let qty = form.quantity.unwrap_or(1).clamp(1, 999);
    match db::get_product(&state.db, form.product_id) {
        Ok(Some(p)) if p.is_active && p.in_stock() => {
            let item = CartItem {
                product_id: p.id,
                name: p.name.clone(),
                slug: p.slug.clone(),
                price: p.price,
                image: p.main_image(),
                quantity: qty,
                max_stock: if p.track_stock { p.stock } else { 0 },
            };
            cart::add_item(&session, item).await;
        }
        _ => {}
    }
    let to = form
        .redirect
        .filter(|p| p.starts_with('/') && !p.starts_with("//") && !p.contains(['\r', '\n']))
        .unwrap_or_else(|| "/cart".into());
    Redirect::to(&to).into_response()
}

#[derive(Deserialize)]
pub struct UpdateForm {
    pub product_id: i64,
    pub quantity: i64,
}

pub async fn update_cart(session: Session, Form(form): Form<UpdateForm>) -> Response {
    cart::update_quantity(&session, form.product_id, form.quantity).await;
    Redirect::to("/cart").into_response()
}

#[derive(Deserialize)]
pub struct RemoveForm {
    pub product_id: i64,
}

pub async fn remove_from_cart(session: Session, Form(form): Form<RemoveForm>) -> Response {
    cart::remove_item(&session, form.product_id).await;
    Redirect::to("/cart").into_response()
}

pub async fn cart_count_api(session: Session) -> Json<serde_json::Value> {
    let items = cart::get_cart(&session).await;
    Json(json!({ "count": cart::cart_count(&items), "subtotal": cart::cart_subtotal(&items) }))
}
