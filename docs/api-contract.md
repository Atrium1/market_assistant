# Market Assistant — API Contract

Desktop V1 uses Tauri `invoke` commands. A future HTTP/mobile API should mirror these names and payloads.

Money is always integer **kuruş** (`price_cents`, `total_cents`, …). UI formats as `₺X,XX`.

## Products

| Command | Future REST | Notes |
|---------|-------------|--------|
| `products_list` | `GET /products` | Active products for current store |
| `products_get_by_barcode` | `GET /products?barcode=` | 404 → not found |
| `products_create` | `POST /products` | Body: `{ barcode, name, price_cents }` |
| `products_update` | `PATCH /products/:id` | Body: `{ id, barcode?, name?, price_cents? }` |
| `products_deactivate` | `DELETE /products/:id` | Soft delete (`is_active = 0`) |

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

| Command | Future REST | Notes |
|---------|-------------|--------|
| `sales_checkout` | `POST /sales` | Persists sale + line snapshots in one transaction |

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

### Checkout response

```json
{
  "sale": {
    "id": "uuid",
    "store_id": "store_default",
    "total_cents": 3000,
    "status": "completed",
    "paid_at": "RFC3339",
    "created_at": "RFC3339"
  },
  "items": [ /* sale_items with name_snapshot */ ]
}
```

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
- `Bu özellik henüz hazır değil`
- `Sepet boş`

## Expansion mapping

| Feature | Contract addition |
|---------|-------------------|
| Stock | Adjust inside `sales_checkout`; `GET /inventory`, `POST /inventory/adjust` |
| Daily reports | `GET /sales?from=&to=`, `GET /reports/daily` |
| Returns | Implement `returns_create` + migration `002` |
| Mobile | Same JSON over HTTPS + auth; Postgres behind repositories |
