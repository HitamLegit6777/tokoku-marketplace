// First-run setup wizard. Lets a non-technical owner configure the store name,
// contact, theme, and payment in one guided form without touching code.
use crate::db;
use crate::filters;
use crate::models::Settings;
use crate::services::themes::{self, Theme};
use crate::state::AppState;
use askama::Template;
use axum::extract::{Form, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use serde_json::json;

use super::server_error;
use super::TemplateResponse;

#[derive(Template)]
#[template(path = "setup/wizard.html")]
struct WizardTemplate {
    settings: Settings,
    themes: &'static [Theme],
    theme_css: String,
}

pub async fn wizard(State(state): State<AppState>) -> Response {
    let settings = state.settings();
    let theme = themes::get_theme(&settings.theme);
    let theme_css = theme.to_css_vars();
    WizardTemplate { settings, themes: themes::THEMES, theme_css }.page()
}

#[derive(Deserialize)]
pub struct WizardForm {
    pub store_name: String,
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub whatsapp: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub instagram: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub shipping_flat: Option<i64>,
    #[serde(default)]
    pub shipping_free_min: Option<i64>,
    // payment
    #[serde(default)]
    pub bank_name: String,
    #[serde(default)]
    pub bank_account_number: String,
    #[serde(default)]
    pub bank_account_holder: String,
    #[serde(default)]
    pub bank_transfer_enabled: Option<String>,
    #[serde(default)]
    pub qris_enabled: Option<String>,
    #[serde(default)]
    pub cod_enabled: Option<String>,
    #[serde(default)]
    pub ewallet_enabled: Option<String>,
    #[serde(default)]
    pub ewallet_name: String,
    #[serde(default)]
    pub ewallet_number: String,
}

fn checkbox(v: &Option<String>) -> bool {
    matches!(v.as_deref(), Some("on") | Some("true") | Some("1"))
}

pub async fn save_wizard(State(state): State<AppState>, Form(form): Form<WizardForm>) -> Response {
    let mut settings = state.settings();
    settings.store_name = if form.store_name.trim().is_empty() { "Toko Saya".into() } else { form.store_name.trim().to_string() };
    settings.tagline = form.tagline.trim().to_string();
    settings.description = form.description.trim().to_string();
    if !form.theme.is_empty() {
        settings.theme = form.theme;
    }
    settings.whatsapp = form.whatsapp.trim().to_string();
    settings.phone = form.whatsapp.trim().to_string();
    settings.email = form.email.trim().to_string();
    settings.address = form.address.trim().to_string();
    settings.instagram = form.instagram.trim().to_string();
    settings.currency = if form.currency.trim().is_empty() { "Rp".into() } else { form.currency.trim().to_string() };
    settings.shipping_flat = form.shipping_flat.unwrap_or(0).max(0);
    settings.shipping_free_min = form.shipping_free_min.unwrap_or(0).max(0);

    let payment = json!({
        "bank_transfer_enabled": checkbox(&form.bank_transfer_enabled),
        "bank_name": form.bank_name.trim(),
        "bank_account_number": form.bank_account_number.trim(),
        "bank_account_holder": form.bank_account_holder.trim(),
        "qris_enabled": checkbox(&form.qris_enabled),
        "cod_enabled": checkbox(&form.cod_enabled),
        "ewallet_enabled": checkbox(&form.ewallet_enabled),
        "ewallet_name": form.ewallet_name.trim(),
        "ewallet_number": form.ewallet_number.trim(),
        "midtrans_enabled": false
    });
    settings.payment_config = payment.to_string();
    settings.setup_done = true;

    if let Err(e) = db::update_settings(&state.db, &settings) {
        return server_error(e);
    }
    Redirect::to("/admin").into_response()
}
