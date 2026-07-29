// Database access layer: connection pool, migrations, and typed queries.
use crate::models::*;
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row};

pub type Db = Pool<SqliteConnectionManager>;

/// Open (or create) the database, enable WAL, and run migrations.
pub fn init(path: &str) -> Result<Db> {
    let manager = SqliteConnectionManager::file(path).with_init(|c| {
        c.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )
    });
    let pool = Pool::builder().max_size(8).build(manager)?;
    {
        let conn = pool.get()?;
        conn.execute_batch(include_str!("../migrations/001_init.sql"))?;
        // Ensure the singleton settings row exists.
        conn.execute(
            "INSERT OR IGNORE INTO settings (id) VALUES (1)",
            [],
        )?;
    }
    Ok(pool)
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub fn get_settings(db: &Db) -> Result<Settings> {
    let conn = db.get()?;
    let s = conn.query_row(
        "SELECT store_name, tagline, description, logo_url, favicon_url, hero_image_url,
                theme, phone, whatsapp, email, address, instagram, facebook, tiktok,
                currency, shipping_flat, shipping_free_min, tax_percent, payment_config,
                meta_keywords, announcement, announcement_on, setup_done
         FROM settings WHERE id = 1",
        [],
        |r| {
            Ok(Settings {
                store_name: r.get(0)?,
                tagline: r.get(1)?,
                description: r.get(2)?,
                logo_url: r.get(3)?,
                favicon_url: r.get(4)?,
                hero_image_url: r.get(5)?,
                theme: r.get(6)?,
                phone: r.get(7)?,
                whatsapp: r.get(8)?,
                email: r.get(9)?,
                address: r.get(10)?,
                instagram: r.get(11)?,
                facebook: r.get(12)?,
                tiktok: r.get(13)?,
                currency: r.get(14)?,
                shipping_flat: r.get(15)?,
                shipping_free_min: r.get(16)?,
                tax_percent: r.get(17)?,
                payment_config: r.get(18)?,
                meta_keywords: r.get(19)?,
                announcement: r.get(20)?,
                announcement_on: r.get::<_, i64>(21)? != 0,
                setup_done: r.get::<_, i64>(22)? != 0,
            })
        },
    )?;
    Ok(s)
}

pub fn update_settings(db: &Db, s: &Settings) -> Result<()> {
    let conn = db.get()?;
    conn.execute(
        "UPDATE settings SET
            store_name=?1, tagline=?2, description=?3, logo_url=?4, favicon_url=?5,
            hero_image_url=?6, theme=?7, phone=?8, whatsapp=?9, email=?10, address=?11,
            instagram=?12, facebook=?13, tiktok=?14, currency=?15, shipping_flat=?16,
            shipping_free_min=?17, tax_percent=?18, payment_config=?19, meta_keywords=?20,
            announcement=?21, announcement_on=?22, setup_done=?23, updated_at=datetime('now')
         WHERE id = 1",
        params![
            s.store_name, s.tagline, s.description, s.logo_url, s.favicon_url,
            s.hero_image_url, s.theme, s.phone, s.whatsapp, s.email, s.address,
            s.instagram, s.facebook, s.tiktok, s.currency, s.shipping_flat,
            s.shipping_free_min, s.tax_percent, s.payment_config, s.meta_keywords,
            s.announcement, s.announcement_on as i64, s.setup_done as i64,
        ],
    )?;
    Ok(())
}

pub fn set_theme(db: &Db, theme: &str) -> Result<()> {
    let conn = db.get()?;
    conn.execute("UPDATE settings SET theme=?1 WHERE id=1", params![theme])?;
    Ok(())
}

pub fn mark_setup_done(db: &Db) -> Result<()> {
    let conn = db.get()?;
    conn.execute("UPDATE settings SET setup_done=1 WHERE id=1", [])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Row mappers
// ---------------------------------------------------------------------------

fn map_product(r: &Row) -> rusqlite::Result<Product> {
    Ok(Product {
        id: r.get("id")?,
        name: r.get("name")?,
        slug: r.get("slug")?,
        sku: r.get("sku")?,
        description: r.get("description")?,
        short_desc: r.get("short_desc")?,
        price: r.get("price")?,
        compare_price: r.get("compare_price")?,
        cost_price: r.get("cost_price")?,
        stock: r.get("stock")?,
        track_stock: r.get::<_, i64>("track_stock")? != 0,
        weight_grams: r.get("weight_grams")?,
        category_id: r.get("category_id")?,
        images: r.get("images")?,
        is_active: r.get::<_, i64>("is_active")? != 0,
        is_featured: r.get::<_, i64>("is_featured")? != 0,
        views: r.get("views")?,
        sold: r.get("sold")?,
        tags: r.get("tags")?,
        created_at: r.get("created_at")?,
        category_name: String::new(),
        avg_rating: 0.0,
        review_count: 0,
    })
}

const PRODUCT_COLS: &str = "id, name, slug, sku, description, short_desc, price, compare_price, \
    cost_price, stock, track_stock, weight_grams, category_id, images, is_active, is_featured, \
    views, sold, tags, created_at";

// ---------------------------------------------------------------------------
// Categories
// ---------------------------------------------------------------------------

pub fn list_categories(db: &Db) -> Result<Vec<Category>> {
    let conn = db.get()?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, c.slug, c.description, c.image_url, c.icon, c.sort_order,
                (SELECT COUNT(*) FROM products p WHERE p.category_id = c.id AND p.is_active = 1) AS pc
         FROM categories c ORDER BY c.sort_order, c.name",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Category {
            id: r.get(0)?,
            name: r.get(1)?,
            slug: r.get(2)?,
            description: r.get(3)?,
            image_url: r.get(4)?,
            icon: r.get(5)?,
            sort_order: r.get(6)?,
            product_count: r.get(7)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_category_by_slug(db: &Db, slug: &str) -> Result<Option<Category>> {
    let conn = db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, slug, description, image_url, icon, sort_order FROM categories WHERE slug=?1",
    )?;
    let mut rows = stmt.query_map(params![slug], |r| {
        Ok(Category {
            id: r.get(0)?, name: r.get(1)?, slug: r.get(2)?, description: r.get(3)?,
            image_url: r.get(4)?, icon: r.get(5)?, sort_order: r.get(6)?, product_count: 0,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn create_category(db: &Db, name: &str, slug: &str, desc: &str, icon: &str, image: &str) -> Result<i64> {
    let conn = db.get()?;
    conn.execute(
        "INSERT INTO categories (name, slug, description, icon, image_url) VALUES (?1,?2,?3,?4,?5)",
        params![name, slug, desc, icon, image],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_category(db: &Db, id: i64, name: &str, slug: &str, desc: &str, icon: &str, image: &str) -> Result<()> {
    let conn = db.get()?;
    conn.execute(
        "UPDATE categories SET name=?1, slug=?2, description=?3, icon=?4, image_url=?5 WHERE id=?6",
        params![name, slug, desc, icon, image, id],
    )?;
    Ok(())
}

pub fn delete_category(db: &Db, id: i64) -> Result<()> {
    let conn = db.get()?;
    conn.execute("DELETE FROM categories WHERE id=?1", params![id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Products
// ---------------------------------------------------------------------------

/// Filters for storefront product listing.
#[derive(Default)]
pub struct ProductFilter {
    pub category_id: Option<i64>,
    pub search: Option<String>,
    pub featured_only: bool,
    pub sort: String, // newest|price_asc|price_desc|popular|name
    pub limit: i64,
    pub offset: i64,
    pub include_inactive: bool,
}

pub fn list_products(db: &Db, f: &ProductFilter) -> Result<Vec<Product>> {
    let conn = db.get()?;
    let mut where_parts: Vec<String> = Vec::new();
    if !f.include_inactive {
        where_parts.push("is_active = 1".into());
    }
    if let Some(cid) = f.category_id {
        where_parts.push(format!("category_id = {}", cid));
    }
    if f.featured_only {
        where_parts.push("is_featured = 1".into());
    }
    if let Some(s) = &f.search {
        let esc = s.replace('\'', "''");
        where_parts.push(format!(
            "(name LIKE '%{0}%' OR description LIKE '%{0}%' OR tags LIKE '%{0}%' OR sku LIKE '%{0}%')",
            esc
        ));
    }
    let where_clause = if where_parts.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_parts.join(" AND "))
    };
    let order = match f.sort.as_str() {
        "price_asc" => "price ASC",
        "price_desc" => "price DESC",
        "popular" => "sold DESC, views DESC",
        "name" => "name ASC",
        _ => "created_at DESC",
    };
    let limit_clause = if f.limit > 0 {
        format!("LIMIT {} OFFSET {}", f.limit, f.offset)
    } else {
        String::new()
    };
    let sql = format!(
        "SELECT {cols}, COALESCE((SELECT name FROM categories c WHERE c.id = products.category_id),'') AS category_name,
                COALESCE((SELECT AVG(rating) FROM reviews rv WHERE rv.product_id = products.id AND rv.is_approved=1),0) AS avg_rating,
                (SELECT COUNT(*) FROM reviews rv WHERE rv.product_id = products.id AND rv.is_approved=1) AS review_count
         FROM products {where_clause} ORDER BY {order} {limit_clause}",
        cols = PRODUCT_COLS
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        let mut p = map_product(r)?;
        p.category_name = r.get("category_name")?;
        p.avg_rating = r.get("avg_rating")?;
        p.review_count = r.get("review_count")?;
        Ok(p)
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn count_products(db: &Db, f: &ProductFilter) -> Result<i64> {
    let conn = db.get()?;
    let mut where_parts: Vec<String> = Vec::new();
    if !f.include_inactive {
        where_parts.push("is_active = 1".into());
    }
    if let Some(cid) = f.category_id {
        where_parts.push(format!("category_id = {}", cid));
    }
    if f.featured_only {
        where_parts.push("is_featured = 1".into());
    }
    if let Some(s) = &f.search {
        let esc = s.replace('\'', "''");
        where_parts.push(format!(
            "(name LIKE '%{0}%' OR description LIKE '%{0}%' OR tags LIKE '%{0}%')",
            esc
        ));
    }
    let where_clause = if where_parts.is_empty() { String::new() } else { format!("WHERE {}", where_parts.join(" AND ")) };
    let sql = format!("SELECT COUNT(*) FROM products {}", where_clause);
    let n: i64 = conn.query_row(&sql, [], |r| r.get(0))?;
    Ok(n)
}

pub fn get_product_by_slug(db: &Db, slug: &str) -> Result<Option<Product>> {
    let conn = db.get()?;
    let sql = format!(
        "SELECT {cols}, COALESCE((SELECT name FROM categories c WHERE c.id = products.category_id),'') AS category_name,
                COALESCE((SELECT AVG(rating) FROM reviews rv WHERE rv.product_id = products.id AND rv.is_approved=1),0) AS avg_rating,
                (SELECT COUNT(*) FROM reviews rv WHERE rv.product_id = products.id AND rv.is_approved=1) AS review_count
         FROM products WHERE slug=?1",
        cols = PRODUCT_COLS
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map(params![slug], |r| {
        let mut p = map_product(r)?;
        p.category_name = r.get("category_name")?;
        p.avg_rating = r.get("avg_rating")?;
        p.review_count = r.get("review_count")?;
        Ok(p)
    })?;
    Ok(rows.next().transpose()?)
}

pub fn get_product(db: &Db, id: i64) -> Result<Option<Product>> {
    let conn = db.get()?;
    let sql = format!("SELECT {cols} FROM products WHERE id=?1", cols = PRODUCT_COLS);
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map(params![id], |r| map_product(r))?;
    Ok(rows.next().transpose()?)
}

pub fn increment_views(db: &Db, id: i64) {
    if let Ok(conn) = db.get() {
        let _ = conn.execute("UPDATE products SET views = views + 1 WHERE id=?1", params![id]);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn upsert_product(db: &Db, p: &Product, id: Option<i64>) -> Result<i64> {
    let conn = db.get()?;
    match id {
        Some(pid) => {
            conn.execute(
                "UPDATE products SET name=?1, slug=?2, sku=?3, description=?4, short_desc=?5,
                    price=?6, compare_price=?7, cost_price=?8, stock=?9, track_stock=?10,
                    weight_grams=?11, category_id=?12, images=?13, is_active=?14, is_featured=?15,
                    tags=?16, updated_at=datetime('now') WHERE id=?17",
                params![
                    p.name, p.slug, p.sku, p.description, p.short_desc, p.price, p.compare_price,
                    p.cost_price, p.stock, p.track_stock as i64, p.weight_grams, p.category_id,
                    p.images, p.is_active as i64, p.is_featured as i64, p.tags, pid
                ],
            )?;
            Ok(pid)
        }
        None => {
            conn.execute(
                "INSERT INTO products (name, slug, sku, description, short_desc, price, compare_price,
                    cost_price, stock, track_stock, weight_grams, category_id, images, is_active,
                    is_featured, tags)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                params![
                    p.name, p.slug, p.sku, p.description, p.short_desc, p.price, p.compare_price,
                    p.cost_price, p.stock, p.track_stock as i64, p.weight_grams, p.category_id,
                    p.images, p.is_active as i64, p.is_featured as i64, p.tags
                ],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn delete_product(db: &Db, id: i64) -> Result<()> {
    let conn = db.get()?;
    conn.execute("DELETE FROM products WHERE id=?1", params![id])?;
    Ok(())
}

pub fn toggle_product_active(db: &Db, id: i64) -> Result<()> {
    let conn = db.get()?;
    conn.execute("UPDATE products SET is_active = 1 - is_active WHERE id=?1", params![id])?;
    Ok(())
}

pub fn related_products(db: &Db, product: &Product, limit: i64) -> Result<Vec<Product>> {
    let f = ProductFilter {
        category_id: product.category_id,
        limit,
        sort: "popular".into(),
        ..Default::default()
    };
    let mut list = list_products(db, &f)?;
    list.retain(|p| p.id != product.id);
    list.truncate(limit as usize);
    Ok(list)
}

// ---------------------------------------------------------------------------
// Orders
// ---------------------------------------------------------------------------

fn map_order(r: &Row) -> rusqlite::Result<Order> {
    Ok(Order {
        id: r.get("id")?,
        order_number: r.get("order_number")?,
        customer_name: r.get("customer_name")?,
        customer_phone: r.get("customer_phone")?,
        customer_email: r.get("customer_email")?,
        shipping_address: r.get("shipping_address")?,
        shipping_city: r.get("shipping_city")?,
        shipping_note: r.get("shipping_note")?,
        subtotal: r.get("subtotal")?,
        shipping_cost: r.get("shipping_cost")?,
        tax: r.get("tax")?,
        discount: r.get("discount")?,
        total: r.get("total")?,
        payment_method: r.get("payment_method")?,
        payment_status: r.get("payment_status")?,
        order_status: r.get("order_status")?,
        payment_ref: r.get("payment_ref")?,
        payment_proof: r.get("payment_proof")?,
        tracking_number: r.get("tracking_number")?,
        notes: r.get("notes")?,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
        items: Vec::new(),
    })
}

const ORDER_COLS: &str = "id, order_number, customer_name, customer_phone, customer_email, \
    shipping_address, shipping_city, shipping_note, subtotal, shipping_cost, tax, discount, total, \
    payment_method, payment_status, order_status, payment_ref, payment_proof, tracking_number, \
    notes, created_at, updated_at";

pub fn create_order(db: &Db, order: &Order, items: &[OrderItem]) -> Result<i64> {
    let mut conn = db.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO orders (order_number, customer_name, customer_phone, customer_email,
            shipping_address, shipping_city, shipping_note, subtotal, shipping_cost, tax,
            discount, total, payment_method, payment_status, order_status, payment_ref, notes)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
        params![
            order.order_number, order.customer_name, order.customer_phone, order.customer_email,
            order.shipping_address, order.shipping_city, order.shipping_note, order.subtotal,
            order.shipping_cost, order.tax, order.discount, order.total, order.payment_method,
            order.payment_status, order.order_status, order.payment_ref, order.notes
        ],
    )?;
    let order_id = tx.last_insert_rowid();
    for it in items {
        tx.execute(
            "INSERT INTO order_items (order_id, product_id, product_name, product_image, price, quantity, subtotal)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![order_id, it.product_id, it.product_name, it.product_image, it.price, it.quantity, it.subtotal],
        )?;
        // decrement stock & bump sold count
        if let Some(pid) = it.product_id {
            tx.execute(
                "UPDATE products SET stock = MAX(0, stock - ?1), sold = sold + ?1 WHERE id = ?2 AND track_stock = 1",
                params![it.quantity, pid],
            )?;
            tx.execute(
                "UPDATE products SET sold = sold + ?1 WHERE id = ?2 AND track_stock = 0",
                params![it.quantity, pid],
            )?;
        }
    }
    tx.commit()?;
    Ok(order_id)
}

pub fn get_order(db: &Db, id: i64) -> Result<Option<Order>> {
    let conn = db.get()?;
    let sql = format!("SELECT {cols} FROM orders WHERE id=?1", cols = ORDER_COLS);
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map(params![id], |r| map_order(r))?;
    let mut order = match rows.next().transpose()? {
        Some(o) => o,
        None => return Ok(None),
    };
    order.items = get_order_items(db, order.id)?;
    Ok(Some(order))
}

pub fn get_order_by_number(db: &Db, number: &str) -> Result<Option<Order>> {
    let conn = db.get()?;
    let sql = format!("SELECT {cols} FROM orders WHERE order_number=?1", cols = ORDER_COLS);
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map(params![number], |r| map_order(r))?;
    let mut order = match rows.next().transpose()? {
        Some(o) => o,
        None => return Ok(None),
    };
    order.items = get_order_items(db, order.id)?;
    Ok(Some(order))
}

fn get_order_items(db: &Db, order_id: i64) -> Result<Vec<OrderItem>> {
    let conn = db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, order_id, product_id, product_name, product_image, price, quantity, subtotal
         FROM order_items WHERE order_id=?1",
    )?;
    let rows = stmt.query_map(params![order_id], |r| {
        Ok(OrderItem {
            id: r.get(0)?, order_id: r.get(1)?, product_id: r.get(2)?, product_name: r.get(3)?,
            product_image: r.get(4)?, price: r.get(5)?, quantity: r.get(6)?, subtotal: r.get(7)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_orders(db: &Db, status: Option<&str>, limit: i64, offset: i64) -> Result<Vec<Order>> {
    let conn = db.get()?;
    let where_clause = match status {
        Some(s) if !s.is_empty() && s != "all" => format!("WHERE order_status = '{}'", s.replace('\'', "")),
        _ => String::new(),
    };
    let sql = format!(
        "SELECT {cols} FROM orders {where_clause} ORDER BY created_at DESC LIMIT {limit} OFFSET {offset}",
        cols = ORDER_COLS
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| map_order(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn update_order_status(db: &Db, id: i64, order_status: &str, payment_status: &str, tracking: &str) -> Result<()> {
    let conn = db.get()?;
    conn.execute(
        "UPDATE orders SET order_status=?1, payment_status=?2, tracking_number=?3, updated_at=datetime('now') WHERE id=?4",
        params![order_status, payment_status, tracking, id],
    )?;
    Ok(())
}

pub fn set_payment_proof(db: &Db, order_number: &str, proof_url: &str) -> Result<()> {
    let conn = db.get()?;
    conn.execute(
        "UPDATE orders SET payment_proof=?1, updated_at=datetime('now') WHERE order_number=?2",
        params![proof_url, order_number],
    )?;
    Ok(())
}

pub fn count_orders(db: &Db, status: Option<&str>) -> Result<i64> {
    let conn = db.get()?;
    let where_clause = match status {
        Some(s) if !s.is_empty() && s != "all" => format!("WHERE order_status = '{}'", s.replace('\'', "")),
        _ => String::new(),
    };
    let sql = format!("SELECT COUNT(*) FROM orders {}", where_clause);
    Ok(conn.query_row(&sql, [], |r| r.get(0))?)
}

// ---------------------------------------------------------------------------
// Users / auth
// ---------------------------------------------------------------------------

pub fn create_user(db: &Db, username: &str, email: &str, password_hash: &str) -> Result<i64> {
    let conn = db.get()?;
    conn.execute(
        "INSERT INTO users (username, email, password_hash) VALUES (?1,?2,?3)",
        params![username, email, password_hash],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_user_by_username(db: &Db, username: &str) -> Result<Option<(i64, String, String)>> {
    let conn = db.get()?;
    let mut stmt = conn.prepare("SELECT id, username, password_hash FROM users WHERE username=?1")?;
    let mut rows = stmt.query_map(params![username], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    })?;
    Ok(rows.next().transpose()?)
}

pub fn count_users(db: &Db) -> Result<i64> {
    let conn = db.get()?;
    Ok(conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?)
}

// ---------------------------------------------------------------------------
// Coupons
// ---------------------------------------------------------------------------

fn map_coupon(r: &Row) -> rusqlite::Result<Coupon> {
    Ok(Coupon {
        id: r.get("id")?,
        code: r.get("code")?,
        r#type: r.get("type")?,
        value: r.get("value")?,
        min_purchase: r.get("min_purchase")?,
        max_uses: r.get("max_uses")?,
        used_count: r.get("used_count")?,
        expires_at: r.get("expires_at")?,
        is_active: r.get::<_, i64>("is_active")? != 0,
    })
}

pub fn list_coupons(db: &Db) -> Result<Vec<Coupon>> {
    let conn = db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, code, type, value, min_purchase, max_uses, used_count, expires_at, is_active
         FROM coupons ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |r| map_coupon(r))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_coupon_by_code(db: &Db, code: &str) -> Result<Option<Coupon>> {
    let conn = db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, code, type, value, min_purchase, max_uses, used_count, expires_at, is_active
         FROM coupons WHERE code=?1 COLLATE NOCASE",
    )?;
    let mut rows = stmt.query_map(params![code], |r| map_coupon(r))?;
    Ok(rows.next().transpose()?)
}

pub fn create_coupon(db: &Db, c: &Coupon) -> Result<i64> {
    let conn = db.get()?;
    conn.execute(
        "INSERT INTO coupons (code, type, value, min_purchase, max_uses, expires_at, is_active)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![c.code, c.r#type, c.value, c.min_purchase, c.max_uses, c.expires_at, c.is_active as i64],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_coupon(db: &Db, id: i64, c: &Coupon) -> Result<()> {
    let conn = db.get()?;
    conn.execute(
        "UPDATE coupons SET code=?1, type=?2, value=?3, min_purchase=?4, max_uses=?5 WHERE id=?6",
        params![c.code, c.r#type, c.value, c.min_purchase, c.max_uses, id],
    )?;
    Ok(())
}

pub fn increment_coupon_use(db: &Db, id: i64) -> Result<()> {
    let conn = db.get()?;
    conn.execute("UPDATE coupons SET used_count = used_count + 1 WHERE id=?1", params![id])?;
    Ok(())
}

pub fn delete_coupon(db: &Db, id: i64) -> Result<()> {
    let conn = db.get()?;
    conn.execute("DELETE FROM coupons WHERE id=?1", params![id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Reviews
// ---------------------------------------------------------------------------

pub fn list_reviews_for_product(db: &Db, product_id: i64) -> Result<Vec<Review>> {
    let conn = db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, product_id, customer_name, rating, comment, is_approved, created_at
         FROM reviews WHERE product_id=?1 AND is_approved=1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![product_id], |r| {
        Ok(Review {
            id: r.get(0)?, product_id: r.get(1)?, customer_name: r.get(2)?, rating: r.get(3)?,
            comment: r.get(4)?, is_approved: r.get::<_, i64>(5)? != 0, created_at: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn create_review(db: &Db, product_id: i64, name: &str, rating: i64, comment: &str) -> Result<i64> {
    let conn = db.get()?;
    conn.execute(
        "INSERT INTO reviews (product_id, customer_name, rating, comment) VALUES (?1,?2,?3,?4)",
        params![product_id, name, rating, comment],
    )?;
    Ok(conn.last_insert_rowid())
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

pub fn list_pages(db: &Db, footer_only: bool) -> Result<Vec<Page>> {
    let conn = db.get()?;
    let where_clause = if footer_only { "WHERE is_published=1 AND show_in_footer=1" } else { "" };
    let sql = format!(
        "SELECT id, title, slug, content, is_published, show_in_footer FROM pages {} ORDER BY id",
        where_clause
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(Page {
            id: r.get(0)?, title: r.get(1)?, slug: r.get(2)?, content: r.get(3)?,
            is_published: r.get::<_, i64>(4)? != 0, show_in_footer: r.get::<_, i64>(5)? != 0,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_page_by_slug(db: &Db, slug: &str) -> Result<Option<Page>> {
    let conn = db.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, title, slug, content, is_published, show_in_footer FROM pages WHERE slug=?1",
    )?;
    let mut rows = stmt.query_map(params![slug], |r| {
        Ok(Page {
            id: r.get(0)?, title: r.get(1)?, slug: r.get(2)?, content: r.get(3)?,
            is_published: r.get::<_, i64>(4)? != 0, show_in_footer: r.get::<_, i64>(5)? != 0,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn upsert_page(db: &Db, title: &str, slug: &str, content: &str, id: Option<i64>) -> Result<i64> {
    let conn = db.get()?;
    match id {
        Some(pid) => {
            conn.execute(
                "UPDATE pages SET title=?1, slug=?2, content=?3 WHERE id=?4",
                params![title, slug, content, pid],
            )?;
            Ok(pid)
        }
        None => {
            conn.execute(
                "INSERT INTO pages (title, slug, content) VALUES (?1,?2,?3)",
                params![title, slug, content],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn delete_page(db: &Db, id: i64) -> Result<()> {
    let conn = db.get()?;
    conn.execute("DELETE FROM pages WHERE id=?1", params![id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Banners
// ---------------------------------------------------------------------------

pub fn list_banners(db: &Db, active_only: bool) -> Result<Vec<Banner>> {
    let conn = db.get()?;
    let where_clause = if active_only { "WHERE is_active=1" } else { "" };
    let sql = format!(
        "SELECT id, title, subtitle, image_url, link_url, button_text, sort_order, is_active
         FROM banners {} ORDER BY sort_order, id",
        where_clause
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(Banner {
            id: r.get(0)?, title: r.get(1)?, subtitle: r.get(2)?, image_url: r.get(3)?,
            link_url: r.get(4)?, button_text: r.get(5)?, sort_order: r.get(6)?,
            is_active: r.get::<_, i64>(7)? != 0,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn create_banner(db: &Db, b: &Banner) -> Result<i64> {
    let conn = db.get()?;
    conn.execute(
        "INSERT INTO banners (title, subtitle, image_url, link_url, button_text, sort_order, is_active)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![b.title, b.subtitle, b.image_url, b.link_url, b.button_text, b.sort_order, b.is_active as i64],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_banner(db: &Db, id: i64, b: &Banner) -> Result<()> {
    let conn = db.get()?;
    conn.execute(
        "UPDATE banners SET title=?1, subtitle=?2, image_url=?3, link_url=?4, button_text=?5, sort_order=?6 WHERE id=?7",
        params![b.title, b.subtitle, b.image_url, b.link_url, b.button_text, b.sort_order, id],
    )?;
    Ok(())
}

pub fn delete_banner(db: &Db, id: i64) -> Result<()> {
    let conn = db.get()?;
    conn.execute("DELETE FROM banners WHERE id=?1", params![id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Dashboard statistics
// ---------------------------------------------------------------------------

#[derive(Debug, Default, serde::Serialize)]
pub struct DashboardStats {
    pub total_products: i64,
    pub active_products: i64,
    pub total_orders: i64,
    pub new_orders: i64,
    pub total_revenue: i64,
    pub revenue_today: i64,
    pub pending_payments: i64,
    pub low_stock: i64,
    pub total_customers: i64,
    pub recent_orders: Vec<Order>,
    pub top_products: Vec<Product>,
    pub sales_last_7_days: Vec<(String, i64)>,
}

pub fn dashboard_stats(db: &Db) -> Result<DashboardStats> {
    let conn = db.get()?;
    let mut s = DashboardStats::default();

    s.total_products = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0))?;
    s.active_products = conn.query_row("SELECT COUNT(*) FROM products WHERE is_active=1", [], |r| r.get(0))?;
    s.total_orders = conn.query_row("SELECT COUNT(*) FROM orders", [], |r| r.get(0))?;
    s.new_orders = conn.query_row("SELECT COUNT(*) FROM orders WHERE order_status='new'", [], |r| r.get(0))?;
    s.total_revenue = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM orders WHERE payment_status='paid'", [], |r| r.get(0),
    )?;
    s.revenue_today = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM orders WHERE payment_status='paid' AND date(created_at)=date('now')",
        [], |r| r.get(0),
    )?;
    s.pending_payments = conn.query_row(
        "SELECT COUNT(*) FROM orders WHERE payment_status='pending'", [], |r| r.get(0),
    )?;
    s.low_stock = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE track_stock=1 AND stock <= 5 AND is_active=1", [], |r| r.get(0),
    )?;
    s.total_customers = conn.query_row(
        "SELECT COUNT(DISTINCT customer_phone) FROM orders", [], |r| r.get(0),
    )?;

    // Recent orders
    s.recent_orders = list_orders(db, None, 8, 0)?;

    // Top products by sales
    let f = ProductFilter { sort: "popular".into(), limit: 5, include_inactive: true, ..Default::default() };
    s.top_products = list_products(db, &f)?;

    // Sales for the last 7 days
    let mut stmt = conn.prepare(
        "SELECT date(created_at) AS d, COALESCE(SUM(total),0) AS t
         FROM orders WHERE payment_status='paid' AND created_at >= date('now','-6 days')
         GROUP BY date(created_at) ORDER BY d",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    s.sales_last_7_days = rows.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(s)
}

// ---------------------------------------------------------------------------
// Financial report (Laporan Keuangan)
// ---------------------------------------------------------------------------

/// A single row in a labelled money breakdown (e.g. by payment method).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct FinanceRow {
    pub label: String,
    pub orders: i64,
    pub amount: i64,
}

/// A product ranked by revenue contribution.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct FinanceProduct {
    pub name: String,
    pub qty: i64,
    pub revenue: i64,
    pub cost: i64,
    pub profit: i64,
}

/// Aggregated finances for a reporting period. All money values are in the
/// store's minor-unit-free integer currency (whole Rupiah).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct FinanceReport {
    pub period: String,      // label describing the range (e.g. "30 hari terakhir")
    pub start_date: String,  // inclusive yyyy-mm-dd
    pub end_date: String,    // inclusive yyyy-mm-dd
    // Headline figures (paid orders only)
    pub gross_revenue: i64,  // sum of order totals
    pub product_sales: i64,  // sum of item subtotals (before shipping/tax/discount)
    pub shipping_income: i64,
    pub tax_collected: i64,
    pub discounts_given: i64,
    pub cogs: i64,           // cost of goods sold (from product cost_price)
    pub gross_profit: i64,   // product_sales - cogs
    pub net_profit: i64,     // gross_revenue - cogs (shipping/tax pass-through kept)
    pub paid_orders: i64,
    pub avg_order_value: i64,
    pub margin_percent: f64, // gross_profit / product_sales * 100
    // Operational context
    pub pending_amount: i64, // value of unpaid (pending) orders
    pub pending_orders: i64,
    pub refunded_amount: i64,
    pub refunded_orders: i64,
    pub cancelled_orders: i64,
    // Breakdowns
    pub monthly: Vec<FinanceRow>,        // last up-to-12 months revenue
    pub by_payment: Vec<FinanceRow>,     // paid revenue grouped by method
    pub top_products: Vec<FinanceProduct>,
    pub max_monthly: i64,                // for chart scaling
}

/// Build a financial report over the last `days` days (inclusive of today).
/// COGS is derived by joining sold items to the current product `cost_price`;
/// items whose product was deleted contribute zero cost.
pub fn finance_report(db: &Db, days: i64) -> Result<FinanceReport> {
    let conn = db.get()?;
    let days = days.clamp(1, 3650);
    let mut r = FinanceReport::default();
    r.period = match days {
        7 => "7 hari terakhir".into(),
        30 => "30 hari terakhir".into(),
        90 => "90 hari terakhir".into(),
        365 => "1 tahun terakhir".into(),
        n => format!("{n} hari terakhir"),
    };
    // Window bound as a datetime string usable in comparisons.
    let since = format!("-{} days", days - 1);
    r.start_date = conn.query_row(
        "SELECT date('now', ?1)", params![since], |row| row.get(0),
    )?;
    r.end_date = conn.query_row("SELECT date('now')", [], |row| row.get(0))?;

    // Headline totals over paid orders in the window.
    let paid_where = "payment_status='paid' AND date(created_at) >= date('now', ?1)";
    // Same predicate but with every column qualified, for queries that JOIN
    // orders to other tables (otherwise `created_at` is ambiguous).
    let paid_where_o = "o.payment_status='paid' AND date(o.created_at) >= date('now', ?1)";
    r.gross_revenue = conn.query_row(
        &format!("SELECT COALESCE(SUM(total),0) FROM orders WHERE {paid_where}"),
        params![since], |row| row.get(0),
    )?;
    r.shipping_income = conn.query_row(
        &format!("SELECT COALESCE(SUM(shipping_cost),0) FROM orders WHERE {paid_where}"),
        params![since], |row| row.get(0),
    )?;
    r.tax_collected = conn.query_row(
        &format!("SELECT COALESCE(SUM(tax),0) FROM orders WHERE {paid_where}"),
        params![since], |row| row.get(0),
    )?;
    r.discounts_given = conn.query_row(
        &format!("SELECT COALESCE(SUM(discount),0) FROM orders WHERE {paid_where}"),
        params![since], |row| row.get(0),
    )?;
    r.paid_orders = conn.query_row(
        &format!("SELECT COUNT(*) FROM orders WHERE {paid_where}"),
        params![since], |row| row.get(0),
    )?;

    // Product sales (item subtotals) and COGS via join to current product cost.
    r.product_sales = conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(oi.subtotal),0) FROM order_items oi \
             JOIN orders o ON o.id = oi.order_id WHERE {paid_where_o}"
        ),
        params![since], |row| row.get(0),
    )?;
    r.cogs = conn.query_row(
        &format!(
            "SELECT COALESCE(SUM(oi.quantity * COALESCE(p.cost_price,0)),0) \
             FROM order_items oi JOIN orders o ON o.id = oi.order_id \
             LEFT JOIN products p ON p.id = oi.product_id WHERE {paid_where_o}"
        ),
        params![since], |row| row.get(0),
    )?;

    r.gross_profit = r.product_sales - r.cogs;
    r.net_profit = r.gross_revenue - r.cogs;
    r.avg_order_value = if r.paid_orders > 0 { r.gross_revenue / r.paid_orders } else { 0 };
    r.margin_percent = if r.product_sales > 0 {
        (r.gross_profit as f64) / (r.product_sales as f64) * 100.0
    } else { 0.0 };

    // Outstanding / non-revenue context (whole history, not windowed, so the
    // owner always sees money still owed and losses).
    r.pending_amount = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM orders WHERE payment_status='pending'", [], |row| row.get(0),
    )?;
    r.pending_orders = conn.query_row(
        "SELECT COUNT(*) FROM orders WHERE payment_status='pending'", [], |row| row.get(0),
    )?;
    r.refunded_amount = conn.query_row(
        "SELECT COALESCE(SUM(total),0) FROM orders WHERE payment_status='refunded'", [], |row| row.get(0),
    )?;
    r.refunded_orders = conn.query_row(
        "SELECT COUNT(*) FROM orders WHERE payment_status='refunded'", [], |row| row.get(0),
    )?;
    r.cancelled_orders = conn.query_row(
        "SELECT COUNT(*) FROM orders WHERE order_status='cancelled'", [], |row| row.get(0),
    )?;

    // Monthly revenue for the last 12 months (paid orders).
    let mut stmt = conn.prepare(
        "SELECT strftime('%Y-%m', created_at) AS m, COUNT(*) AS c, COALESCE(SUM(total),0) AS t \
         FROM orders WHERE payment_status='paid' AND created_at >= date('now','-11 months','start of month') \
         GROUP BY m ORDER BY m",
    )?;
    r.monthly = stmt
        .query_map([], |row| Ok(FinanceRow {
            label: row.get::<_, String>(0)?,
            orders: row.get(1)?,
            amount: row.get(2)?,
        }))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    r.max_monthly = r.monthly.iter().map(|m| m.amount).max().unwrap_or(0).max(1);

    // Revenue grouped by payment method (paid orders in the window).
    let mut stmt = conn.prepare(&format!(
        "SELECT payment_method, COUNT(*), COALESCE(SUM(total),0) \
         FROM orders WHERE {paid_where} GROUP BY payment_method ORDER BY 3 DESC"
    ))?;
    r.by_payment = stmt
        .query_map(params![since], |row| Ok(FinanceRow {
            label: row.get::<_, String>(0)?,
            orders: row.get(1)?,
            amount: row.get(2)?,
        }))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Top products by revenue (paid orders in the window).
    let mut stmt = conn.prepare(&format!(
        "SELECT oi.product_name, SUM(oi.quantity) AS q, SUM(oi.subtotal) AS rev, \
                SUM(oi.quantity * COALESCE(p.cost_price,0)) AS cost \
         FROM order_items oi JOIN orders o ON o.id = oi.order_id \
         LEFT JOIN products p ON p.id = oi.product_id \
         WHERE {paid_where_o} GROUP BY oi.product_name ORDER BY rev DESC LIMIT 10"
    ))?;
    r.top_products = stmt
        .query_map(params![since], |row| {
            let revenue: i64 = row.get(2)?;
            let cost: i64 = row.get(3)?;
            Ok(FinanceProduct {
                name: row.get::<_, String>(0)?,
                qty: row.get(1)?,
                revenue,
                cost,
                profit: revenue - cost,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(r)
}

/// Count of distinct products (used to decide whether to seed demo data).
pub fn is_empty_store(db: &Db) -> Result<bool> {
    Ok(count_products(db, &ProductFilter { include_inactive: true, ..Default::default() })? == 0)
}
