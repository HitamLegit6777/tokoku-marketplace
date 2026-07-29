// Checkout & order handlers: checkout page, order placement, confirmation,
// payment proof upload, order tracking, and the Midtrans payment webhook.
use crate::filters;
use crate::db;
use crate::models::{Coupon, Order, OrderItem, PaymentConfig, Settings};
use crate::services::{cart, helpers, payment};
use crate::services::payment::PaymentMethod;
use crate::state::AppState;
use askama::Template;
use axum::extract::{Form, Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;
use tower_sessions::Session;

use super::store::Shared;
use super::TemplateResponse;
use super::{redirect, server_error};

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "store/checkout.html")]
struct CheckoutTemplate {
    s: Shared,
    items: Vec<crate::models::CartItem>,
    subtotal: i64,
    shipping: i64,
    tax: i64,
    total: i64,
    methods: Vec<PaymentMethod>,
    free_shipping: bool,
}

#[derive(Template)]
#[template(path = "store/order_confirmation.html")]
struct OrderConfirmationTemplate {
    s: Shared,
    order: Order,
    payment: PaymentConfig,
    method_label: String,
    wa_link: String,
    is_paid: bool,
    show_bank: bool,
    show_qris: bool,
    show_ewallet: bool,
    show_cod: bool,
}

#[derive(Template)]
#[template(path = "store/track.html")]
struct TrackTemplate {
    s: Shared,
    order: Option<Order>,
    searched: bool,
    query: String,
}

// ---------------------------------------------------------------------------
// Checkout page
// ---------------------------------------------------------------------------

pub async fn checkout_page(State(state): State<AppState>, session: Session) -> Response {
    let s = Shared::build(&state, &session).await;
    let items = cart::get_cart(&session).await;
    if items.is_empty() {
        return Redirect::to("/cart").into_response();
    }
    let subtotal = cart::cart_subtotal(&items);
    let settings = &s.settings;
    let free_shipping = settings.shipping_free_min > 0 && subtotal >= settings.shipping_free_min;
    let shipping = if free_shipping { 0 } else { settings.shipping_flat };
    let tax = ((subtotal as f64) * settings.tax_percent / 100.0).round() as i64;
    let total = subtotal + shipping + tax;
    let methods = payment::enabled_methods(&settings.payment());

    CheckoutTemplate { s, items, subtotal, shipping, tax, total, methods, free_shipping }.page()
}

// ---------------------------------------------------------------------------
// Place order
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CheckoutForm {
    pub customer_name: String,
    pub customer_phone: String,
    #[serde(default)]
    pub customer_email: String,
    pub shipping_address: String,
    #[serde(default)]
    pub shipping_city: String,
    #[serde(default)]
    pub shipping_note: String,
    pub payment_method: String,
    #[serde(default)]
    pub coupon_code: String,
}

pub async fn place_order(State(state): State<AppState>, session: Session, Form(form): Form<CheckoutForm>) -> Response {
    let items = cart::get_cart(&session).await;
    if items.is_empty() {
        return Redirect::to("/cart").into_response();
    }
    if form.customer_name.trim().is_empty() || form.customer_phone.trim().is_empty() || form.shipping_address.trim().is_empty() {
        return redirect("/checkout?error=incomplete");
    }

    let settings: Settings = state.settings();
    let subtotal = cart::cart_subtotal(&items);
    let free_shipping = settings.shipping_free_min > 0 && subtotal >= settings.shipping_free_min;
    let shipping = if free_shipping { 0 } else { settings.shipping_flat };
    let tax = ((subtotal as f64) * settings.tax_percent / 100.0).round() as i64;

    // Apply coupon if valid
    let mut discount = 0i64;
    let mut coupon_used: Option<Coupon> = None;
    if !form.coupon_code.trim().is_empty() {
        if let Ok(Some(c)) = db::get_coupon_by_code(&state.db, form.coupon_code.trim()) {
            if is_coupon_valid(&c, subtotal) {
                discount = compute_discount(&c, subtotal);
                coupon_used = Some(c);
            }
        }
    }

    let total = (subtotal + shipping + tax - discount).max(0);

    let order = Order {
        order_number: helpers::generate_order_number(),
        customer_name: form.customer_name.trim().to_string(),
        customer_phone: form.customer_phone.trim().to_string(),
        customer_email: form.customer_email.trim().to_string(),
        shipping_address: form.shipping_address.trim().to_string(),
        shipping_city: form.shipping_city.trim().to_string(),
        shipping_note: form.shipping_note.trim().to_string(),
        subtotal,
        shipping_cost: shipping,
        tax,
        discount,
        total,
        payment_method: form.payment_method.clone(),
        payment_status: "pending".into(),
        order_status: "new".into(),
        ..Default::default()
    };

    let order_items: Vec<OrderItem> = items
        .iter()
        .map(|c| OrderItem {
            product_id: Some(c.product_id),
            product_name: c.name.clone(),
            product_image: c.image.clone(),
            price: c.price,
            quantity: c.quantity,
            subtotal: c.subtotal(),
            ..Default::default()
        })
        .collect();

    let order_id = match db::create_order(&state.db, &order, &order_items) {
        Ok(id) => id,
        Err(e) => return server_error(e),
    };
    if let Some(c) = coupon_used {
        let _ = db::increment_coupon_use(&state.db, c.id);
    }
    cart::clear_cart(&session).await;

    // If Midtrans selected, create snap transaction and redirect to payment page.
    if form.payment_method == "midtrans" {
        let cfg = settings.payment();
        if let Ok(Some(full_order)) = db::get_order(&state.db, order_id) {
            match payment::create_midtrans_snap(&cfg, &full_order, &state.base_url).await {
                Ok(snap) => {
                    let _ = db::update_order_status(&state.db, order_id, "new", "pending", "");
                    if !snap.redirect_url.is_empty() {
                        return Redirect::to(&snap.redirect_url).into_response();
                    }
                }
                Err(e) => tracing::error!("Midtrans snap failed: {e}"),
            }
        }
    }

    Redirect::to(&format!("/order/{}", order.order_number)).into_response()
}

fn is_coupon_valid(c: &Coupon, subtotal: i64) -> bool {
    if !c.is_active {
        return false;
    }
    if c.min_purchase > 0 && subtotal < c.min_purchase {
        return false;
    }
    if c.max_uses > 0 && c.used_count >= c.max_uses {
        return false;
    }
    true
}

fn compute_discount(c: &Coupon, subtotal: i64) -> i64 {
    let d = if c.r#type == "percent" {
        (subtotal * c.value) / 100
    } else {
        c.value
    };
    d.min(subtotal)
}

// ---------------------------------------------------------------------------
// Order confirmation
// ---------------------------------------------------------------------------

pub async fn order_confirmation(State(state): State<AppState>, session: Session, Path(number): Path<String>) -> Response {
    let s = Shared::build(&state, &session).await;
    match db::get_order_by_number(&state.db, &number) {
        Ok(Some(order)) => {
            let payment = s.settings.payment();
            let method_label = method_label(&order.payment_method, &payment);
            let is_paid = order.payment_status == "paid";
            let msg = format!(
                "Halo, saya ingin konfirmasi pesanan {} atas nama {} dengan total Rp {}.",
                order.order_number, order.customer_name, helpers::format_rupiah(order.total)
            );
            let wa_link = helpers::whatsapp_link(&s.settings.whatsapp, &msg);
            let show_bank = order.payment_method == "transfer";
            let show_qris = order.payment_method == "qris";
            let show_ewallet = order.payment_method == "ewallet";
            let show_cod = order.payment_method == "cod";
            OrderConfirmationTemplate {
                s, order, payment, method_label, wa_link, is_paid,
                show_bank, show_qris, show_ewallet, show_cod,
            }
            .page()
        }
        Ok(None) => super::store::not_found(State(state), session).await,
        Err(e) => server_error(e),
    }
}

fn method_label(method: &str, cfg: &PaymentConfig) -> String {
    match method {
        "transfer" => "Transfer Bank".to_string(),
        "qris" => "QRIS".to_string(),
        "ewallet" => if cfg.ewallet_name.is_empty() { "E-Wallet".to_string() } else { cfg.ewallet_name.clone() },
        "cod" => "Bayar di Tempat (COD)".to_string(),
        "midtrans" => "Pembayaran Online".to_string(),
        _ => method.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Payment proof upload
// ---------------------------------------------------------------------------

pub async fn upload_proof(State(state): State<AppState>, Path(number): Path<String>, mut multipart: axum::extract::Multipart) -> Response {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "proof" {
            let filename = field.file_name().unwrap_or("proof.jpg").to_string();
            if let Ok(bytes) = field.bytes().await {
                match crate::services::upload::save_image(&bytes, &filename) {
                    Ok(url) => {
                        let _ = db::set_payment_proof(&state.db, &number, &url);
                    }
                    Err(e) => tracing::error!("proof upload failed: {e}"),
                }
            }
        }
    }
    Redirect::to(&format!("/order/{}?uploaded=1", number)).into_response()
}

// ---------------------------------------------------------------------------
// Order tracking
// ---------------------------------------------------------------------------

pub async fn track_form(State(state): State<AppState>, session: Session) -> Response {
    let s = Shared::build(&state, &session).await;
    TrackTemplate { s, order: None, searched: false, query: String::new() }.page()
}

#[derive(Deserialize)]
pub struct TrackQuery {
    pub order: Option<String>,
}

pub async fn track_result(State(state): State<AppState>, session: Session, Query(q): Query<TrackQuery>) -> Response {
    let s = Shared::build(&state, &session).await;
    let query = q.order.unwrap_or_default();
    let order = if query.trim().is_empty() {
        None
    } else {
        db::get_order_by_number(&state.db, query.trim()).ok().flatten()
    };
    TrackTemplate { s, order, searched: true, query }.page()
}

// ---------------------------------------------------------------------------
// Midtrans webhook
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct MidtransNotification {
    pub order_id: String,
    pub status_code: String,
    pub gross_amount: String,
    pub signature_key: String,
    pub transaction_status: String,
    #[serde(default)]
    pub fraud_status: String,
}

pub async fn midtrans_webhook(State(state): State<AppState>, Json(notif): Json<MidtransNotification>) -> Response {
    let settings = state.settings();
    let cfg = settings.payment();
    if !payment::verify_midtrans_signature(
        &cfg.midtrans_server_key,
        &notif.order_id,
        &notif.status_code,
        &notif.gross_amount,
        &notif.signature_key,
    ) {
        tracing::warn!("Midtrans webhook: invalid signature for {}", notif.order_id);
        return (axum::http::StatusCode::FORBIDDEN, "invalid signature").into_response();
    }

    let payment_status = payment::map_midtrans_status(&notif.transaction_status, &notif.fraud_status);
    if let Ok(Some(order)) = db::get_order_by_number(&state.db, &notif.order_id) {
        let order_status = if payment_status == "paid" { "processing" } else { order.order_status.as_str() };
        let _ = db::update_order_status(&state.db, order.id, order_status, payment_status, &order.tracking_number);
        tracing::info!("Midtrans webhook: order {} -> {}", notif.order_id, payment_status);
    }
    (axum::http::StatusCode::OK, "ok").into_response()
}
