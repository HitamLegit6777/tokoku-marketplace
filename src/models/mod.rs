// Data models mirroring the SQLite schema.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub store_name: String,
    pub tagline: String,
    pub description: String,
    pub logo_url: String,
    pub favicon_url: String,
    pub hero_image_url: String,
    pub theme: String,
    pub phone: String,
    pub whatsapp: String,
    pub email: String,
    pub address: String,
    pub instagram: String,
    pub facebook: String,
    pub tiktok: String,
    pub currency: String,
    pub shipping_flat: i64,
    pub shipping_free_min: i64,
    pub tax_percent: f64,
    pub payment_config: String,
    pub meta_keywords: String,
    pub announcement: String,
    pub announcement_on: bool,
    pub setup_done: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            store_name: "Toko Saya".into(),
            tagline: "Belanja mudah, aman, dan terpercaya".into(),
            description: String::new(),
            logo_url: String::new(),
            favicon_url: String::new(),
            hero_image_url: String::new(),
            theme: "sunset".into(),
            phone: String::new(),
            whatsapp: String::new(),
            email: String::new(),
            address: String::new(),
            instagram: String::new(),
            facebook: String::new(),
            tiktok: String::new(),
            currency: "Rp".into(),
            shipping_flat: 0,
            shipping_free_min: 0,
            tax_percent: 0.0,
            payment_config: "{}".into(),
            meta_keywords: String::new(),
            announcement: String::new(),
            announcement_on: false,
            setup_done: false,
        }
    }
}

impl Settings {
    /// Parse the JSON payment_config into a structured value.
    pub fn payment(&self) -> PaymentConfig {
        serde_json::from_str(&self.payment_config).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaymentConfig {
    #[serde(default)]
    pub bank_transfer_enabled: bool,
    #[serde(default)]
    pub bank_name: String,
    #[serde(default)]
    pub bank_account_number: String,
    #[serde(default)]
    pub bank_account_holder: String,
    #[serde(default)]
    pub qris_enabled: bool,
    #[serde(default)]
    pub qris_image_url: String,
    #[serde(default)]
    pub cod_enabled: bool,
    #[serde(default)]
    pub ewallet_enabled: bool,
    #[serde(default)]
    pub ewallet_name: String,
    #[serde(default)]
    pub ewallet_number: String,
    #[serde(default)]
    pub midtrans_enabled: bool,
    #[serde(default)]
    pub midtrans_server_key: String,
    #[serde(default)]
    pub midtrans_client_key: String,
    #[serde(default)]
    pub midtrans_production: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub image_url: String,
    pub icon: String,
    pub sort_order: i64,
    #[serde(default)]
    pub product_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub sku: String,
    pub description: String,
    pub short_desc: String,
    pub price: i64,
    pub compare_price: i64,
    pub cost_price: i64,
    pub stock: i64,
    pub track_stock: bool,
    pub weight_grams: i64,
    pub category_id: Option<i64>,
    pub images: String, // JSON array
    pub is_active: bool,
    pub is_featured: bool,
    pub views: i64,
    pub sold: i64,
    pub tags: String,
    pub created_at: String,
    // joined / computed
    #[serde(default)]
    pub category_name: String,
    #[serde(default)]
    pub avg_rating: f64,
    #[serde(default)]
    pub review_count: i64,
}

impl Product {
    /// Deserialize the image list; falls back to a placeholder when empty.
    pub fn image_list(&self) -> Vec<String> {
        let v: Vec<String> = serde_json::from_str(&self.images).unwrap_or_default();
        if v.is_empty() {
            vec!["/static/img/placeholder.svg".to_string()]
        } else {
            v
        }
    }

    pub fn main_image(&self) -> String {
        self.image_list().into_iter().next().unwrap_or_else(|| "/static/img/placeholder.svg".into())
    }

    pub fn has_discount(&self) -> bool {
        self.compare_price > self.price && self.compare_price > 0
    }

    pub fn discount_percent(&self) -> i64 {
        if self.has_discount() {
            ((self.compare_price - self.price) * 100) / self.compare_price
        } else {
            0
        }
    }

    pub fn in_stock(&self) -> bool {
        !self.track_stock || self.stock > 0
    }

    pub fn tag_list(&self) -> Vec<String> {
        self.tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    }

    /// Whether this product belongs to the given category id (template helper).
    /// Accepts `i64` or `&i64` since Askama passes references for field access.
    pub fn is_category<T: std::borrow::Borrow<i64>>(&self, id: T) -> bool {
        self.category_id == Some(*id.borrow())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Order {
    pub id: i64,
    pub order_number: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: String,
    pub shipping_address: String,
    pub shipping_city: String,
    pub shipping_note: String,
    pub subtotal: i64,
    pub shipping_cost: i64,
    pub tax: i64,
    pub discount: i64,
    pub total: i64,
    pub payment_method: String,
    pub payment_status: String,
    pub order_status: String,
    pub payment_ref: String,
    pub payment_proof: String,
    pub tracking_number: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub items: Vec<OrderItem>,
}

impl Order {
    pub fn status_label(&self) -> &'static str {
        match self.order_status.as_str() {
            "new" => "Pesanan Baru",
            "processing" => "Diproses",
            "shipped" => "Dikirim",
            "completed" => "Selesai",
            "cancelled" => "Dibatalkan",
            _ => "Tidak diketahui",
        }
    }

    pub fn payment_label(&self) -> &'static str {
        match self.payment_status.as_str() {
            "pending" => "Menunggu Pembayaran",
            "paid" => "Sudah Dibayar",
            "failed" => "Gagal",
            "refunded" => "Dikembalikan",
            _ => "Tidak diketahui",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderItem {
    pub id: i64,
    pub order_id: i64,
    pub product_id: Option<i64>,
    pub product_name: String,
    pub product_image: String,
    pub price: i64,
    pub quantity: i64,
    pub subtotal: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Coupon {
    pub id: i64,
    pub code: String,
    pub r#type: String,
    pub value: i64,
    pub min_purchase: i64,
    pub max_uses: i64,
    pub used_count: i64,
    pub expires_at: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Review {
    pub id: i64,
    pub product_id: i64,
    pub customer_name: String,
    pub rating: i64,
    pub comment: String,
    pub is_approved: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub is_published: bool,
    pub show_in_footer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Banner {
    pub id: i64,
    pub title: String,
    pub subtitle: String,
    pub image_url: String,
    pub link_url: String,
    pub button_text: String,
    pub sort_order: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: String,
}

/// Item stored in the shopping cart (session).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: i64,
    pub name: String,
    pub slug: String,
    pub price: i64,
    pub image: String,
    pub quantity: i64,
    pub max_stock: i64,
}

impl CartItem {
    pub fn subtotal(&self) -> i64 {
        self.price * self.quantity
    }
}
