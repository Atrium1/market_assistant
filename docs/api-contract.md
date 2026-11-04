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

## Expansion mapping

| Feature | Contract addition |
|---------|-------------------|
| Stock | Adjust inside `sales_checkout`; `GET /inventory`, `POST /inventory/adjust` |
| Daily reports | `GET /sales?from=&to=`, `GET /reports/daily` |
| Returns | Implement `returns_create` + migration |
| Mobile | Same JSON over HTTPS + auth; Postgres behind repositories |
