import { invoke } from "@tauri-apps/api/core";
import type {
  CheckoutItemInput,
  CheckoutResult,
  Product,
} from "../types";

function asErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Beklenmeyen bir hata oluştu";
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    throw new Error(asErrorMessage(error));
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

  salesCheckout: (items: CheckoutItemInput[]) =>
    call<CheckoutResult>("sales_checkout", { input: { items } }),

  returnsCreate: () => call<void>("returns_create"),
};
