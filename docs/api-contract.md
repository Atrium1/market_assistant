# Market Assistant — API Contract

Desktop V1 uses Tauri `invoke` commands. A future HTTP/mobile API should mirror these names and payloads.

Money is always integer **kuruş** (`price_cents`, `total_cents`, …). UI formats as `₺X,XX`.

## Auth

Owner session is **in-memory** (cleared on app restart). Idle timeout: 15 minutes. Mutating product/settings calls refresh activity.

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
| `sales_checkout` | `POST /sales` | public | Persists sale + line snapshots in one transaction |

### Checkout request

```json
{
  "items": [
    {
      "product_id": "uuid",
      "barcode": "869…",
      "name": "Su 0.5L",
      "unit_price_cents": 1500,
      "qty": 2
    }
  ]
}
```

Cart itself is **UI state only**. Persistence boundary is checkout.

## Price check

No dedicated command. Use `products_get_by_barcode`. UI must **not** auto-add to cart in Fiyat sor mode.

## Returns (stub)

| Command | Future REST | Notes |
|---------|-------------|--------|
| `returns_create` | `POST /returns` | Currently returns `not_implemented` |

## Errors

Commands return a string error message suitable for Turkish UI toasts, e.g.:

- `Ürün bulunamadı`
- `Bu barkod zaten kayıtlı`
- `Kullanıcı adı veya şifre hatalı`
- `Bu işlem için sahip girişi gerekli.`
- `Oturum süresi doldu. Lütfen tekrar giriş yapın.`
- `Sepet boş`
- `Bu dosya geçerli bir Market Asistanı yedeği değil`
- `Bu yedek daha yeni bir sürümden; önce uygulamayı güncelleyin`
- `Uygulanacak geçerli satır yok`

## Expansion mapping

| Feature | Contract addition |
|---------|-------------------|
| Stock | Adjust inside `sales_checkout`; `GET /inventory`, `POST /inventory/adjust` |
| Daily reports | `GET /sales?from=&to=`, `GET /reports/daily` |
| Returns | Implement `returns_create` + migration |
| Mobile | Same JSON over HTTPS + auth; Postgres behind repositories |
