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
