// Shopping cart stored in the session as JSON. Kept server-agnostic so it can be
// swapped for a cookie or DB-backed cart later.
use crate::models::CartItem;
use tower_sessions::Session;

const CART_KEY: &str = "cart";

pub async fn get_cart(session: &Session) -> Vec<CartItem> {
    session.get::<Vec<CartItem>>(CART_KEY).await.ok().flatten().unwrap_or_default()
}

pub async fn save_cart(session: &Session, cart: &[CartItem]) {
    let _ = session.insert(CART_KEY, cart).await;
}

pub async fn add_item(session: &Session, item: CartItem) {
    let mut cart = get_cart(session).await;
    if let Some(existing) = cart.iter_mut().find(|c| c.product_id == item.product_id) {
        existing.quantity += item.quantity;
        if existing.max_stock > 0 && existing.quantity > existing.max_stock {
            existing.quantity = existing.max_stock;
        }
    } else {
        cart.push(item);
    }
    save_cart(session, &cart).await;
}

pub async fn update_quantity(session: &Session, product_id: i64, quantity: i64) {
    let mut cart = get_cart(session).await;
    if quantity <= 0 {
        cart.retain(|c| c.product_id != product_id);
    } else if let Some(item) = cart.iter_mut().find(|c| c.product_id == product_id) {
        item.quantity = quantity;
        if item.max_stock > 0 && item.quantity > item.max_stock {
            item.quantity = item.max_stock;
        }
    }
    save_cart(session, &cart).await;
}

pub async fn remove_item(session: &Session, product_id: i64) {
    let mut cart = get_cart(session).await;
    cart.retain(|c| c.product_id != product_id);
    save_cart(session, &cart).await;
}

pub async fn clear_cart(session: &Session) {
    let _ = session.remove::<Vec<CartItem>>(CART_KEY).await;
}

pub fn cart_subtotal(cart: &[CartItem]) -> i64 {
    cart.iter().map(|c| c.subtotal()).sum()
}

pub fn cart_count(cart: &[CartItem]) -> i64 {
    cart.iter().map(|c| c.quantity).sum()
}
