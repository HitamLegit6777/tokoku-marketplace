// Shared application state and template context building.
use crate::db::Db;
use crate::models::{Category, Page, Settings};
use crate::services::themes::{self, Theme};

/// Global application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub base_url: String,
}

impl AppState {
    pub fn settings(&self) -> Settings {
        crate::db::get_settings(&self.db).unwrap_or_default()
    }
}

/// Data every storefront page needs: store settings, active theme, nav categories,
/// footer pages, and cart count. Built once per request.
pub struct StoreContext {
    pub settings: Settings,
    pub theme: &'static Theme,
    pub theme_css: String,
    pub categories: Vec<Category>,
    pub footer_pages: Vec<Page>,
    pub cart_count: i64,
    pub all_themes: &'static [Theme],
}

impl StoreContext {
    pub fn build(state: &AppState, cart_count: i64) -> Self {
        let settings = state.settings();
        let theme = themes::get_theme(&settings.theme);
        let theme_css = theme.to_css_vars();
        let categories = crate::db::list_categories(&state.db).unwrap_or_default();
        let footer_pages = crate::db::list_pages(&state.db, true).unwrap_or_default();
        StoreContext {
            settings,
            theme,
            theme_css,
            categories,
            footer_pages,
            cart_count,
            all_themes: themes::THEMES,
        }
    }

    // Convenience accessors used inside templates.
    pub fn store_name(&self) -> &str { &self.settings.store_name }
    pub fn currency(&self) -> &str { &self.settings.currency }
    pub fn whatsapp_number(&self) -> &str { &self.settings.whatsapp }
}
