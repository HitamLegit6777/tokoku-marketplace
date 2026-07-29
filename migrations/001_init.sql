-- TokoKu database schema
-- SQLite embedded, WAL mode for concurrency

-- Store configuration (single row, no-code editable via admin)
CREATE TABLE IF NOT EXISTS settings (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    store_name      TEXT NOT NULL DEFAULT 'Toko Saya',
    tagline         TEXT NOT NULL DEFAULT 'Belanja mudah, aman, dan terpercaya',
    description     TEXT NOT NULL DEFAULT '',
    logo_url        TEXT NOT NULL DEFAULT '',
    favicon_url     TEXT NOT NULL DEFAULT '',
    hero_image_url  TEXT NOT NULL DEFAULT '',
    theme           TEXT NOT NULL DEFAULT 'sunset',
    -- contact
    phone           TEXT NOT NULL DEFAULT '',
    whatsapp        TEXT NOT NULL DEFAULT '',
    email           TEXT NOT NULL DEFAULT '',
    address         TEXT NOT NULL DEFAULT '',
    instagram       TEXT NOT NULL DEFAULT '',
    facebook        TEXT NOT NULL DEFAULT '',
    tiktok          TEXT NOT NULL DEFAULT '',
    -- commerce
    currency        TEXT NOT NULL DEFAULT 'Rp',
    shipping_flat   INTEGER NOT NULL DEFAULT 0,
    shipping_free_min INTEGER NOT NULL DEFAULT 0,
    tax_percent     REAL NOT NULL DEFAULT 0,
    -- payment config (JSON blob for flexibility)
    payment_config  TEXT NOT NULL DEFAULT '{}',
    -- SEO
    meta_keywords   TEXT NOT NULL DEFAULT '',
    announcement    TEXT NOT NULL DEFAULT '',
    announcement_on INTEGER NOT NULL DEFAULT 0,
    setup_done      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Admin users
CREATE TABLE IF NOT EXISTS users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    username        TEXT NOT NULL UNIQUE,
    email           TEXT NOT NULL DEFAULT '',
    password_hash   TEXT NOT NULL,
    role            TEXT NOT NULL DEFAULT 'admin',
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Product categories
CREATE TABLE IF NOT EXISTS categories (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    slug            TEXT NOT NULL UNIQUE,
    description     TEXT NOT NULL DEFAULT '',
    image_url       TEXT NOT NULL DEFAULT '',
    icon            TEXT NOT NULL DEFAULT '',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Products
CREATE TABLE IF NOT EXISTS products (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    slug            TEXT NOT NULL UNIQUE,
    sku             TEXT NOT NULL DEFAULT '',
    description     TEXT NOT NULL DEFAULT '',
    short_desc      TEXT NOT NULL DEFAULT '',
    price           INTEGER NOT NULL DEFAULT 0,
    compare_price   INTEGER NOT NULL DEFAULT 0,
    cost_price      INTEGER NOT NULL DEFAULT 0,
    stock           INTEGER NOT NULL DEFAULT 0,
    track_stock     INTEGER NOT NULL DEFAULT 1,
    weight_grams    INTEGER NOT NULL DEFAULT 0,
    category_id     INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    images          TEXT NOT NULL DEFAULT '[]',   -- JSON array of urls
    is_active       INTEGER NOT NULL DEFAULT 1,
    is_featured     INTEGER NOT NULL DEFAULT 0,
    views           INTEGER NOT NULL DEFAULT 0,
    sold            INTEGER NOT NULL DEFAULT 0,
    tags            TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category_id);
CREATE INDEX IF NOT EXISTS idx_products_active ON products(is_active);
CREATE INDEX IF NOT EXISTS idx_products_featured ON products(is_featured);

-- Orders
CREATE TABLE IF NOT EXISTS orders (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    order_number    TEXT NOT NULL UNIQUE,
    customer_name   TEXT NOT NULL,
    customer_phone  TEXT NOT NULL,
    customer_email  TEXT NOT NULL DEFAULT '',
    shipping_address TEXT NOT NULL DEFAULT '',
    shipping_city   TEXT NOT NULL DEFAULT '',
    shipping_note   TEXT NOT NULL DEFAULT '',
    subtotal        INTEGER NOT NULL DEFAULT 0,
    shipping_cost   INTEGER NOT NULL DEFAULT 0,
    tax             INTEGER NOT NULL DEFAULT 0,
    discount        INTEGER NOT NULL DEFAULT 0,
    total           INTEGER NOT NULL DEFAULT 0,
    payment_method  TEXT NOT NULL DEFAULT 'transfer',
    payment_status  TEXT NOT NULL DEFAULT 'pending', -- pending|paid|failed|refunded
    order_status    TEXT NOT NULL DEFAULT 'new',     -- new|processing|shipped|completed|cancelled
    payment_ref     TEXT NOT NULL DEFAULT '',
    payment_proof   TEXT NOT NULL DEFAULT '',
    tracking_number TEXT NOT NULL DEFAULT '',
    notes           TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(order_status);
CREATE INDEX IF NOT EXISTS idx_orders_payment ON orders(payment_status);
CREATE INDEX IF NOT EXISTS idx_orders_created ON orders(created_at);

-- Order line items
CREATE TABLE IF NOT EXISTS order_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id        INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id      INTEGER REFERENCES products(id) ON DELETE SET NULL,
    product_name    TEXT NOT NULL,
    product_image   TEXT NOT NULL DEFAULT '',
    price           INTEGER NOT NULL DEFAULT 0,
    quantity        INTEGER NOT NULL DEFAULT 1,
    subtotal        INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_order_items_order ON order_items(order_id);

-- Discount coupons
CREATE TABLE IF NOT EXISTS coupons (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    code            TEXT NOT NULL UNIQUE,
    type            TEXT NOT NULL DEFAULT 'percent',  -- percent|fixed
    value           INTEGER NOT NULL DEFAULT 0,
    min_purchase    INTEGER NOT NULL DEFAULT 0,
    max_uses        INTEGER NOT NULL DEFAULT 0,
    used_count      INTEGER NOT NULL DEFAULT 0,
    expires_at      TEXT,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Customer reviews
CREATE TABLE IF NOT EXISTS reviews (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id      INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    customer_name   TEXT NOT NULL,
    rating          INTEGER NOT NULL DEFAULT 5,
    comment         TEXT NOT NULL DEFAULT '',
    is_approved     INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_reviews_product ON reviews(product_id);

-- Content pages (about, terms, etc)
CREATE TABLE IF NOT EXISTS pages (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    slug            TEXT NOT NULL UNIQUE,
    content         TEXT NOT NULL DEFAULT '',
    is_published    INTEGER NOT NULL DEFAULT 1,
    show_in_footer  INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Banners / promotional slides
CREATE TABLE IF NOT EXISTS banners (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL DEFAULT '',
    subtitle        TEXT NOT NULL DEFAULT '',
    image_url       TEXT NOT NULL DEFAULT '',
    link_url        TEXT NOT NULL DEFAULT '',
    button_text     TEXT NOT NULL DEFAULT 'Belanja Sekarang',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
