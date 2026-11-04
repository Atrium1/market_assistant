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

export type PaymentMethod = "cash" | "card";

export type SaleStatus = "completed" | "voided";

export type CheckoutItemInput = {
  product_id: string;
  qty: number;
  expected_unit_price_cents?: number;
};

export type Sale = {
  id: string;
  store_id: string;
  total_cents: number;
  status: SaleStatus;
  payment_method: PaymentMethod;
  business_day: string;
  receipt_no: number;
  paid_at: string;
  created_at: string;
  voided_at: string | null;
  void_reason: string | null;
};

export type SaleItem = {
  id: string;
  sale_id: string;
  product_id: string | null;
  barcode: string;
  name_snapshot: string;
  unit_price_cents: number;
  qty: number;
  line_total_cents: number;
};

export type CheckoutResult = {
  sale: Sale;
  items: SaleItem[];
};

export type SaleDetail = CheckoutResult;

export type CartChangedDetails = {
  changed: Array<{
    product_id: string;
    name: string;
    old_unit_price_cents: number;
    new_unit_price_cents: number;
  }>;
  removed: Array<{ product_id: string; name: string | null }>;
  current: CartLine[];
};

export type SalesListQuery = {
  from: string;
  to: string;
  status?: SaleStatus;
  payment_method?: PaymentMethod;
  limit?: number;
  cursor?: string;
};

export type SaleListItem = {
  id: string;
  business_day: string;
  receipt_no: number;
  paid_at: string;
  total_cents: number;
  payment_method: PaymentMethod;
  item_count: number;
  status: SaleStatus;
  void_reason: string | null;
};

export type SalesPage = {
  items: SaleListItem[];
  next_cursor: string | null;
};

export type TopProduct = {
  product_id: string | null;
  barcode: string;
  name: string;
  qty: number;
  revenue_cents: number;
};

export type HourBucket = {
  hour: number;
  count: number;
  total_cents: number;
};

export type DailyReport = {
  business_day: string;
  sale_count: number;
  gross_total_cents: number;
  cash_total_cents: number;
  card_total_cents: number;
  voided_count: number;
  voided_total_cents: number;
  returns_total_cents: number;
  net_total_cents: number;
  average_basket_cents: number;
  first_sale_at: string | null;
  last_sale_at: string | null;
  top_by_qty: TopProduct[];
  top_by_revenue: TopProduct[];
  hourly: HourBucket[];
};

export type DayRow = {
  business_day: string;
  sale_count: number;
  net_total_cents: number;
  cash_total_cents: number;
  card_total_cents: number;
  voided_count: number;
};

export type TodaySummary = {
  business_day: string;
  sale_count: number;
  net_total_cents: number;
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
