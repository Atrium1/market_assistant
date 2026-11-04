CREATE TABLE IF NOT EXISTS owner_settings (
    store_id TEXT PRIMARY KEY NOT NULL REFERENCES stores(id),
    shop_username TEXT,
    shop_password_hash TEXT,
    store_display_name TEXT,
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO owner_settings (store_id, shop_username, shop_password_hash, store_display_name, updated_at)
SELECT id, NULL, NULL, name, created_at FROM stores WHERE id = 'store_default';
