export type Product = {
  id: string;
  store_id: string;
  barcode: string;
  name: string;
  price_cents: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
};

export type CheckoutItemInput = {
  product_id: string;
  barcode: string;
  name: string;
  unit_price_cents: number;
  qty: number;
};

export type CheckoutResult = {
  sale: {
    id: string;
    store_id: string;
    total_cents: number;
    status: string;
    paid_at: string;
    created_at: string;
  };
  items: Array<{
    id: string;
    sale_id: string;
    product_id: string | null;
    barcode: string;
    name_snapshot: string;
    unit_price_cents: number;
    qty: number;
    line_total_cents: number;
  }>;
};

export type CartLine = {
  product_id: string;
  barcode: string;
  name: string;
  unit_price_cents: number;
  qty: number;
};

export type RegisterMode = "sale" | "price_check" | "return";

export type SessionStatus = {
  logged_in: boolean;
  via_factory: boolean;
  username_display: string | null;
};

export type SettingsPublic = {
  store_display_name: string | null;
  shop_username: string | null;
  shop_credentials_set: boolean;
};

export type BackupFile = {
  name: string;
  size_bytes: number;
  modified_at: string | null;
};

export type BackupInfo = {
  db_path: string;
  backup_dir: string;
  recent: BackupFile[];
};

export type BackupSummary = {
  product_count: number;
  sale_count: number;
  last_sale_at: string | null;
  schema_versions: string[];
};

export type ImportRowKind = "add" | "update" | "unchanged" | "invalid";

export type ImportRow = {
  row_number: number;
  kind: ImportRowKind;
  barcode: string;
  name: string;
  price_cents: number | null;
  old_name: string | null;
  old_price_cents: number | null;
  reactivate: boolean;
  reason: string | null;
  warning: string | null;
};

export type ImportPreview = {
  file_name: string;
  counts: Record<ImportRowKind, number>;
  add: ImportRow[];
  update: ImportRow[];
  unchanged: ImportRow[];
  invalid: ImportRow[];
  truncated: boolean;
};

export type ImportResult = {
  added: number;
  updated: number;
  reactivated: number;
  skipped: number;
};
