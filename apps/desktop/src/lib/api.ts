import { invoke } from "@tauri-apps/api/core";
import type {
  BackupInfo,
  BackupSummary,
  CartChangedDetails,
  CheckoutItemInput,
  CheckoutResult,
  DailyReport,
  DayRow,
  ImportPreview,
  ImportResult,
  PaymentMethod,
  Product,
  Sale,
  SaleDetail,
  SalesListQuery,
  SalesPage,
  SessionStatus,
  SettingsPublic,
  TodaySummary,
} from "../types";

/** Rust commands reject with `{ code, message, details }`; `message` is Turkish UI text. */
export class ApiError extends Error {
  readonly code: string;
  readonly details: unknown;

  constructor(code: string, message: string, details: unknown = null) {
    super(message);
    this.name = "ApiError";
    this.code = code;
    this.details = details;
  }
}

function toApiError(error: unknown): ApiError {
  if (error && typeof error === "object" && "code" in error && "message" in error) {
    const e = error as { code: unknown; message: unknown; details?: unknown };
    return new ApiError(String(e.code), String(e.message), e.details ?? null);
  }
  if (typeof error === "string") return new ApiError("message", error);
  if (error instanceof Error) return new ApiError("message", error.message);
  return new ApiError("unknown", "Beklenmeyen bir hata oluştu");
}

export function isAuthError(error: unknown): boolean {
  return (
    error instanceof ApiError &&
    (error.code === "owner_required" || error.code === "session_expired")
  );
}

export function cartChangedDetails(error: unknown): CartChangedDetails | null {
  return error instanceof ApiError && error.code === "cart_changed"
    ? (error.details as CartChangedDetails)
    : null;
}

export function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    throw toApiError(error);
  }
}

export const api = {
  productsList: () => call<Product[]>("products_list"),

  productsGetByBarcode: (barcode: string) =>
    call<Product>("products_get_by_barcode", { barcode }),

  productsCreate: (input: {
    barcode: string;
    name: string;
    price_cents: number;
  }) => call<Product>("products_create", { input }),

  productsUpdate: (input: {
    id: string;
    barcode?: string;
    name?: string;
    price_cents?: number;
  }) => call<Product>("products_update", { input }),

  productsDeactivate: (id: string) =>
    call<void>("products_deactivate", { id }),

  salesCheckout: (items: CheckoutItemInput[], payment_method: PaymentMethod) =>
    call<CheckoutResult>("sales_checkout", { input: { items, payment_method } }),

  salesList: (query: SalesListQuery) => call<SalesPage>("sales_list", { query }),

  salesGet: (id: string) => call<SaleDetail>("sales_get", { id }),

  salesVoid: (id: string, reason: string) =>
    call<Sale>("sales_void", { id, reason }),

  reportsDaily: (date: string) => call<DailyReport>("reports_daily", { date }),

  reportsRange: (from: string, to: string) =>
    call<DayRow[]>("reports_range", { from, to }),

  reportsTodaySummary: () => call<TodaySummary>("reports_today_summary"),

  returnsCreate: () => call<void>("returns_create"),

  authLogin: (username: string, password: string) =>
    call<SessionStatus>("auth_login", { input: { username, password } }),

  authLogout: () => call<void>("auth_logout"),

  authSessionStatus: () => call<SessionStatus>("auth_session_status"),

  settingsGet: () => call<SettingsPublic>("settings_get"),

  settingsSetStoreName: (store_display_name: string) =>
    call<void>("settings_set_store_name", {
      input: { store_display_name },
    }),

  settingsSetOwnerCredentials: (input: {
    username: string;
    password: string;
    password_confirm: string;
  }) => call<void>("settings_set_owner_credentials", { input }),

  backupInfo: () => call<BackupInfo>("backup_info"),

  backupCreate: (destPath: string) =>
    call<void>("backup_create", { destPath }),

  backupInspect: (srcPath: string) =>
    call<BackupSummary>("backup_inspect", { srcPath }),

  backupRestore: (srcPath: string) =>
    call<BackupSummary>("backup_restore", { srcPath }),

  productsImportPreview: (path: string) =>
    call<ImportPreview>("products_import_preview", { path }),

  productsImportApply: (path: string) =>
    call<ImportResult>("products_import_apply", { path }),

  productsExport: (destPath: string) =>
    call<number>("products_export", { destPath }),

  productsExportTemplate: (destPath: string) =>
    call<void>("products_export_template", { destPath }),
};
