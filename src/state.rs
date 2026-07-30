// Shared application state and template context building.
use crate::db::Db;
use crate::models::{Category, Page, Settings};
use crate::services::themes::{self, Theme};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const LOGIN_WINDOW: Duration = Duration::from_secs(15 * 60);
const LOGIN_LOCK: Duration = Duration::from_secs(15 * 60);
const LOGIN_MAX_FAILURES: u8 = 5;

#[derive(Clone, Default)]
pub struct LoginLimiter(Arc<Mutex<HashMap<String, LoginAttempt>>>);

struct LoginAttempt {
    failures: u8,
    first_failure: Instant,
    locked_until: Option<Instant>,
}

impl LoginLimiter {
    pub fn is_blocked(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut attempts = self.0.lock().unwrap_or_else(|e| e.into_inner());
        attempts.retain(|_, a| {
            a.locked_until.map_or(
                now.duration_since(a.first_failure) < LOGIN_WINDOW,
                |until| until > now,
            )
        });
        attempts
            .get(key)
            .and_then(|a| a.locked_until)
            .is_some_and(|until| until > now)
    }

    pub fn failure(&self, key: String) {
        let now = Instant::now();
        let mut attempts = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let attempt = attempts.entry(key).or_insert(LoginAttempt {
            failures: 0,
            first_failure: now,
            locked_until: None,
        });
        if now.duration_since(attempt.first_failure) >= LOGIN_WINDOW {
            attempt.failures = 0;
            attempt.first_failure = now;
            attempt.locked_until = None;
        }
        attempt.failures = attempt.failures.saturating_add(1);
        if attempt.failures >= LOGIN_MAX_FAILURES {
            attempt.locked_until = Some(now + LOGIN_LOCK);
        }
    }

    pub fn success(&self, key: &str) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).remove(key);
    }
}

/// Global application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub base_url: String,
    pub login_limiter: LoginLimiter,
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
    /// Absolute site origin (e.g. https://toko.example) for canonical/OG URLs.
    pub base_url: String,
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
            base_url: state.base_url.trim_end_matches('/').to_string(),
        }
    }

    // Convenience accessors used inside templates.
    pub fn store_name(&self) -> &str {
        &self.settings.store_name
    }
    pub fn currency(&self) -> &str {
        &self.settings.currency
    }
    pub fn whatsapp_number(&self) -> &str {
        &self.settings.whatsapp
    }
}
