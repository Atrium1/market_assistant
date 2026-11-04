# Market Assistant — API Contract

Desktop V1 uses Tauri `invoke` commands. A future HTTP/mobile API should mirror these names and payloads.

Money is always integer **kuruş** (`price_cents`, `total_cents`, …). UI formats as `₺X,XX`.

## Auth

Owner session is **in-memory** (cleared on app restart). Idle timeout: 15 minutes. Every owner-only call refreshes activity; a missing or expired session fails with code `owner_required` / `session_expired`.

Login succeeds if credentials match **shop owner** (DB hash) **OR** embedded **factory default** (recovery backdoor; not in DB; not editable in UI).

| Command | Future REST | Auth | Notes |
|---------|-------------|------|--------|
| `auth_login` | `POST /auth/login` | public | `{ username, password }` → session status |
| `auth_logout` | `POST /auth/logout` | public | Clears session |
| `auth_session_status` | `GET /auth/session` | public | `{ logged_in, via_factory, username_display }` |

### Session status shape

```json
{
  "logged_in": true,
  "via_factory": false,
  "username_display": "magaza"
}
```

## Settings (owner session required)

| Command | Future REST | Notes |
|---------|-------------|--------|
| `settings_get` | `GET /settings` | Display name, shop username, `shop_credentials_set` |
| `settings_set_store_name` | `PATCH /settings/store-name` | `{ store_display_name }` |
| `settings_set_owner_credentials` | `PUT /settings/credentials` | `{ username, password, password_confirm }` — no old password required while session active |

Factory credentials are never returned or changeable via API.

## Products

| Command | Future REST | Auth | Notes |
|---------|-------------|------|--------|
| `products_list` | `GET /products` | public | Active products for current store |
| `products_get_by_barcode` | `GET /products?barcode=` | public | Cashier + edit-mode scan |
| `products_create` | `POST /products` | **owner** | Body: `{ barcode, name, price_cents }` |
| `products_update` | `PATCH /products/:id` | **owner** | Body: `{ id, barcode?, name?, price_cents? }` |
| `products_deactivate` | `DELETE /products/:id` | **owner** | Soft delete (`is_active = 0`); UI label **Sil** |

### Product shape

```json
{
  "id": "uuid",
  "store_id": "store_default",
  "barcode": "8690000000001",
  "name": "Su 0.5L",
  "price_cents": 1500,
  "is_active": true,
  "created_at": "RFC3339",
  "updated_at": "RFC3339"
}
```

`products_create` on a barcode that exists but was soft-deleted **reactivates** that row with the new name/price instead of failing on `UNIQUE(store_id, barcode)`.

### Search

No command. `products_list` returns the full active list and the UI filters client-side (Turkish-aware: `İ/ı/ş/ğ/ü/ö/ç` folded, case-insensitive; matches name or barcode substring). A future HTTP API may add `GET /products?q=` for large catalogs.

## Import / export (owner session required)

File paths are absolute paths on the local machine, chosen via the native file dialog. In a future HTTP API the file would be a multipart upload / download instead.

| Command | Future REST | Notes |
|---------|-------------|--------|
| `products_import_preview` | `POST /products/import/preview` | `{ path }` → `ImportPreview`. Reads `.xlsx`/`.xls`/`.csv`, writes nothing |
| `products_import_apply` | `POST /products/import` | `{ path }` → `ImportResult`. Re-reads the file, takes a `pre_import_` safety backup, applies add/update in one transaction |
| `products_export` | `GET /products/export` | `{ destPath }` → number of products written (`.xlsx`, active products) |
| `products_export_template` | `GET /products/template` | `{ destPath }` — empty `.xlsx` template with one sample row |

File format: first sheet (or CSV with `;`, `,` or tab). Header row within the first 5 rows, columns matched case/diacritic-insensitively: **Barkod** (`barkod`, `barkod no`, `barcode`, `ean`), **Ürün adı** (`ürün adı`, `ad`, `isim`, `name`, …), **Fiyat** (`fiyat`, `satış fiyatı`, `price`). Prices accept `12,50`, `12.50`, `1.234,50`, `₺12,50`. CSV is read as UTF-8 (BOM ok) with Windows-1254 fallback. Max 10 MB / 20 000 rows.

Rows are classified as:

- `add` — barcode not in DB
- `update` — barcode exists and name or price differs (or product was deleted → `reactivate: true`)
- `unchanged` — identical to DB
- `invalid` — missing/bad barcode, name or price, or barcode repeated in the file (`reason` explains); skipped on apply

### ImportPreview shape

```json
{
  "file_name": "urunler.xlsx",
  "counts": { "add": 120, "update": 15, "unchanged": 300, "invalid": 2 },
  "add": [ImportRow],
  "update": [ImportRow],
  "unchanged": [ImportRow],
  "invalid": [ImportRow],
  "truncated": false
}
```

Each list holds at most 200 rows (`truncated: true` if any was cut); `counts` are always complete.

```json
{
  "row_number": 7,
  "kind": "update",
  "barcode": "8690000000001",
  "name": "Su 0.5L",
  "price_cents": 1750,
  "old_name": "Su 0.5L",
  "old_price_cents": 1500,
  "reactivate": false,
  "reason": null,
  "warning": null
}
```

### ImportResult shape

```json
{ "added": 120, "updated": 15, "reactivated": 1, "skipped": 2 }
```

## Backup (owner session required)

Backups are full SQLite copies made with the online backup API (safe while the app is running).

| Command | Future REST | Notes |
|---------|-------------|--------|
| `backup_info` | `GET /backups` | `{ db_path, backup_dir, recent: [{ name, size_bytes, modified_at }] }` |
| `backup_create` | `POST /backups` | `{ destPath }` — write a backup file; refuses the live DB path |
| `backup_inspect` | `POST /backups/inspect` | `{ srcPath }` → `BackupSummary`; validates without changing anything |
| `backup_restore` | `POST /backups/restore` | `{ srcPath }` → `BackupSummary`. Takes a `pre_restore_` safety backup, replaces all data, runs migrations, **clears the owner session** |

```json
{
  "product_count": 412,
  "sale_count": 1830,
  "last_sale_at": "RFC3339 | null",
  "schema_versions": ["001_init", "002_owner_settings"]
}
```

Automatic backups live in `<app data>/backups/`:

- `auto_YYYY-MM-DD.db` — first launch of each day, newest 7 kept
- `pre_restore_*.db`, `pre_import_*.db` — before restore / import, newest 5 of each kept

A backup from a **newer** app version (unknown migration) is rejected: `Bu yedek daha yeni bir sürümden; önce uygulamayı güncelleyin`.

## Sales / checkout

| Command | Future REST | Auth | Notes |
|---------|-------------|------|--------|
| `sales_checkout` | `POST /sales` | public | Prices/names from DB; one transaction |
| `sales_list` | `GET /sales?from=&to=&status=&payment_method=&limit=&cursor=` | **owner** | Newest first, keyset pagination |
| `sales_get` | `GET /sales/:id` | **owner** | Sale + line items |
| `sales_void` | `POST /sales/:id/void` | **owner** | `{ id, reason }` — marks voided, never deletes |

Cart itself is **UI state only**. Persistence boundary is checkout.

### Checkout request

```json
{
  "items": [
    { "product_id": "uuid", "qty": 2, "expected_unit_price_cents": 1500 }
  ],
  "payment_method": "cash"
}
```

- Only `product_id` and `qty` are trusted. Price, name and barcode snapshots are read from the DB (active product of the current store) inside the transaction.
- `expected_unit_price_cents` is the price the cashier saw. If it differs from the DB, or a product is inactive/unknown/from another store, nothing is written and the call fails with `cart_changed` (see Errors). Without `expected_unit_price_cents` the DB price is used silently.
- Legacy payloads (`unit_price_cents`, `barcode`, `name`) are accepted; `unit_price_cents` is treated as the expected price.
- Duplicate `product_id` lines are merged. Qty `1..=9999` per product after merging, at most 500 products per sale.
- `payment_method`: `cash` (default) or `card`.

### Sale shape

```json
{
  "id": "uuid",
  "store_id": "store_default",
  "total_cents": 4500,
  "status": "completed | voided",
  "payment_method": "cash | card",
  "business_day": "2026-11-04",
  "receipt_no": 12,
  "paid_at": "2026-11-04T09:30:00.000Z",
  "created_at": "…",
  "voided_at": null,
  "void_reason": null
}
```

- `paid_at` is UTC. `business_day` is the **local** calendar day of the machine at the moment of sale; it is stored, so reports never shift if the timezone changes later. Rows from before migration `003` were backfilled with SQLite `date(paid_at, 'localtime')`.
- `receipt_no` restarts at 1 every business day; shown as `04.11.2026 #0012`.

`sales_checkout` and `sales_get` return `{ sale, items: [{ id, sale_id, product_id, barcode, name_snapshot, unit_price_cents, qty, line_total_cents }] }`.

### Sales list

Request: `{ query: { from: "2026-11-01", to: "2026-11-04", status?: "completed" | "voided", payment_method?: "cash" | "card", limit?: 50 (max 200), cursor?: string } }` — `from`/`to` are inclusive business days.

```json
{
  "items": [
    {
      "id": "uuid",
      "business_day": "2026-11-04",
      "receipt_no": 12,
      "paid_at": "…",
      "total_cents": 4500,
      "payment_method": "card",
      "item_count": 3,
      "status": "completed",
      "void_reason": null
    }
  ],
  "next_cursor": "2026-11-04T09:30:00.000Z|uuid"
}
```

Pass `next_cursor` back as `cursor` for the next page; `null` means no more rows.

### Void

`reason` is required (1–200 characters). A voided sale stays in history, is excluded from net totals, and cannot be voided again (`Bu satış zaten iptal edilmiş`). Any day may be voided; the UI warns if it is not today.

## Reports

| Command | Future REST | Auth | Notes |
|---------|-------------|------|--------|
| `reports_daily` | `GET /reports/daily?date=` | **owner** | `{ date: "YYYY-MM-DD" }` |
| `reports_range` | `GET /reports/daily?from=&to=` | **owner** | One row per day, at most 92 days, days without sales included |
| `reports_today_summary` | `GET /reports/today` | public | Kasa header: today's net total and count |

### Daily report

```json
{
  "business_day": "2026-11-04",
  "sale_count": 3,
  "gross_total_cents": 6000,
  "cash_total_cents": 3000,
  "card_total_cents": 2000,
  "voided_count": 1,
  "voided_total_cents": 1000,
  "returns_total_cents": 0,
  "net_total_cents": 5000,
  "average_basket_cents": 1666,
  "first_sale_at": "…",
  "last_sale_at": "…",
  "top_by_qty": [{ "product_id": "uuid", "barcode": "869…", "name": "Su 0.5L", "qty": 4, "revenue_cents": 2000 }],
  "top_by_revenue": [],
  "hourly": [{ "hour": 9, "count": 2, "total_cents": 4000 }]
}
```

- `gross` = every sale recorded that day; `net = gross − voided − returns`.
- `sale_count`, cash/card totals, first/last sale, top products and `hourly` (24 local-hour buckets) cover non-voided sales only.
- `returns_total_cents` is always 0 until returns are implemented; it is already part of `net`.
- Range rows: `{ business_day, sale_count, net_total_cents, cash_total_cents, card_total_cents, voided_count }`.
- Today summary: `{ business_day, sale_count, net_total_cents }`.

## Price check

No dedicated command. Use `products_get_by_barcode`. UI must **not** auto-add to cart in Fiyat sor mode.

## Returns (stub)

| Command | Future REST | Notes |
|---------|-------------|--------|
| `returns_create` | `POST /returns` | Currently returns `not_implemented` |

## Errors

Failed commands reject with a JSON object (a future HTTP API returns the same body with a 4xx/5xx status):

```json
{ "code": "cart_changed", "message": "Fiyatı değişen ürünler var, sepet güncellendi — toplamı tekrar kontrol edin", "details": { } }
```

`message` is always Turkish text that can be shown as-is. `details` is `null` except for `cart_changed`.

| Code | Meaning |
|------|---------|
| `message` | Validation or business-rule error; show `message` |
| `not_found` | Product not found |
| `not_implemented` | Feature stub |
| `owner_required` | No owner session — open the login dialog and retry |
| `session_expired` | Owner session idle >15 min — open the login dialog and retry |
| `cart_changed` | Checkout rejected; `details` below |
| `db`, `io` | Unexpected storage error |

`cart_changed` details:

```json
{
  "changed": [{ "product_id": "uuid", "name": "Su 0.5L", "old_unit_price_cents": 1500, "new_unit_price_cents": 1750 }],
  "removed": [{ "product_id": "uuid", "name": "Eski ürün" }],
  "current": [{ "product_id": "uuid", "barcode": "869…", "name": "Su 0.5L", "unit_price_cents": 1750, "qty": 2 }]
}
```

The client replaces its cart with `current`, shows the changes, and lets the cashier take payment again.

Example messages:

- `Ürün bulunamadı`
- `Bu barkod zaten kayıtlı`
- `Kullanıcı adı veya şifre hatalı`
- `Bu işlem için sahip girişi gerekli.`
- `Oturum süresi doldu. Lütfen tekrar giriş yapın.`
- `Sepet boş`
- `Bu dosya geçerli bir Market Asistanı yedeği değil`
- `Bu yedek daha yeni bir sürümden; önce uygulamayı güncelleyin`
- `Uygulanacak geçerli satır yok`
- `Satış bulunamadı`
- `Bu satış zaten iptal edilmiş`
- `İptal nedeni yazın`

## Expansion mapping

| Feature | Contract addition |
|---------|-------------------|
| Stock | Deduct inside `sales_checkout` after the price check; restore in `sales_void`; `GET /inventory`, `POST /inventory/adjust` |
| Returns | Implement `returns_create` keyed by `sale_id` + `business_day`; fills `returns_total_cents` |
| Day close | Persist cash counts in a `day_closes` table; `POST /reports/daily/:date/close` |
| Mobile | Same JSON over HTTPS + auth; Postgres behind repositories |
