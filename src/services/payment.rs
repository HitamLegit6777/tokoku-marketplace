// Payment integrations. Supports manual methods (bank transfer, QRIS, e-wallet,
// COD) that need no third party, plus optional Midtrans Snap for automated online
// payments. All manual methods work out of the box with zero external accounts.
use crate::models::{Order, PaymentConfig};
use anyhow::{anyhow, Result};
use sha2::{Digest, Sha512};

/// Which payment methods are enabled, for rendering the checkout UI.
pub fn enabled_methods(cfg: &PaymentConfig) -> Vec<PaymentMethod> {
    let mut methods = Vec::new();
    if cfg.bank_transfer_enabled {
        methods.push(PaymentMethod::new(
            "transfer",
            "Transfer Bank",
            "Transfer manual ke rekening toko, konfirmasi otomatis oleh admin",
            "bank",
        ));
    }
    if cfg.qris_enabled {
        methods.push(PaymentMethod::new(
            "qris",
            "QRIS",
            "Scan QRIS dengan e-wallet atau mobile banking apa pun",
            "qr",
        ));
    }
    if cfg.ewallet_enabled {
        let label = if cfg.ewallet_name.trim().is_empty() {
            "E-Wallet"
        } else {
            cfg.ewallet_name.as_str()
        };
        methods.push(PaymentMethod::new(
            "ewallet",
            label,
            "Bayar via e-wallet (OVO/GoPay/Dana/ShopeePay)",
            "wallet",
        ));
    }
    if cfg.cod_enabled {
        methods.push(PaymentMethod::new(
            "cod",
            "Bayar di Tempat (COD)",
            "Bayar tunai saat barang diterima",
            "cash",
        ));
    }
    if cfg.midtrans_enabled && !cfg.midtrans_server_key.is_empty() {
        methods.push(PaymentMethod::new(
            "midtrans",
            "Pembayaran Online",
            "Kartu kredit, VA, e-wallet & lainnya via Midtrans",
            "card",
        ));
    }
    methods
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PaymentMethod {
    pub id: &'static str,
    pub label: String,
    pub description: &'static str,
    pub icon: &'static str,
}

// Manual conversion since label is now String
impl PaymentMethod {
    fn new(id: &'static str, label: &str, description: &'static str, icon: &'static str) -> Self {
        PaymentMethod {
            id,
            label: label.to_string(),
            description,
            icon,
        }
    }
}

/// Midtrans Snap: create a transaction and return the redirect/snap token.
pub async fn create_midtrans_snap(
    cfg: &PaymentConfig,
    order: &Order,
    base_url: &str,
) -> Result<MidtransSnap> {
    if cfg.midtrans_server_key.is_empty() {
        return Err(anyhow!("Midtrans belum dikonfigurasi"));
    }
    let base = if cfg.midtrans_production {
        "https://app.midtrans.com/snap/v1/transactions"
    } else {
        "https://app.sandbox.midtrans.com/snap/v1/transactions"
    };
    let auth = base64_encode(&format!("{}:", cfg.midtrans_server_key));

    let items: Vec<serde_json::Value> = order
        .items
        .iter()
        .map(|it| {
            serde_json::json!({
                "id": it.product_id.unwrap_or(0).to_string(),
                "price": it.price,
                "quantity": it.quantity,
                "name": truncate_name(&it.product_name),
            })
        })
        .collect();

    let mut all_items = items;
    if order.shipping_cost > 0 {
        all_items.push(serde_json::json!({
            "id": "shipping", "price": order.shipping_cost, "quantity": 1, "name": "Ongkos Kirim"
        }));
    }
    if order.tax > 0 {
        all_items.push(serde_json::json!({
            "id": "tax", "price": order.tax, "quantity": 1, "name": "Pajak"
        }));
    }
    if order.discount > 0 {
        all_items.push(serde_json::json!({
            "id": "discount", "price": -order.discount, "quantity": 1, "name": "Diskon"
        }));
    }

    let payload = serde_json::json!({
        "transaction_details": {
            "order_id": order.order_number,
            "gross_amount": order.total,
        },
        "item_details": all_items,
        "customer_details": {
            "first_name": order.customer_name,
            "phone": order.customer_phone,
            "email": if order.customer_email.is_empty() { "noreply@tokoku.id" } else { &order.customer_email },
        },
        "callbacks": {
            "finish": format!("{}/order/{}", base_url, order.order_number)
        }
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(base)
        .header("Authorization", format!("Basic {auth}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !resp.status().is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Midtrans error: {txt}"));
    }
    let body: serde_json::Value = resp.json().await?;
    let token = body["token"].as_str().unwrap_or_default().to_string();
    let redirect_url = body["redirect_url"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if token.is_empty() {
        return Err(anyhow!("Midtrans tidak mengembalikan token"));
    }
    Ok(MidtransSnap {
        token,
        redirect_url,
    })
}

#[derive(Debug, Clone)]
pub struct MidtransSnap {
    pub token: String,
    pub redirect_url: String,
}

/// Verify a Midtrans webhook signature.
/// signature = sha512(order_id + status_code + gross_amount + server_key)
pub fn verify_midtrans_signature(
    server_key: &str,
    order_id: &str,
    status_code: &str,
    gross_amount: &str,
    signature: &str,
) -> bool {
    let raw = format!("{order_id}{status_code}{gross_amount}{server_key}");
    let mut hasher = Sha512::new();
    hasher.update(raw.as_bytes());
    let digest = hasher.finalize();
    let hex = hex_encode(&digest);
    hex == signature
}

/// Map Midtrans transaction_status to our internal payment_status.
pub fn map_midtrans_status(transaction_status: &str, fraud_status: &str) -> &'static str {
    match transaction_status {
        "capture" => {
            if fraud_status == "accept" {
                "paid"
            } else {
                "pending"
            }
        }
        "settlement" => "paid",
        "pending" => "pending",
        "deny" | "cancel" | "expire" => "failed",
        "refund" | "partial_refund" | "chargeback" => "refunded",
        _ => "pending",
    }
}

fn truncate_name(s: &str) -> String {
    // Midtrans caps item name length at 50 chars
    if s.chars().count() <= 50 {
        s.to_string()
    } else {
        s.chars().take(50).collect()
    }
}

fn base64_encode(s: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(s.as_bytes())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}
