import type { PaymentMethod } from "../types";

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

/** Local calendar day as `YYYY-MM-DD` (same format as `business_day`). */
export function dayKey(d: Date = new Date()): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function parseDayKey(key: string): Date {
  const [y, m, d] = key.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export function addDays(key: string, days: number): string {
  const d = parseDayKey(key);
  d.setDate(d.getDate() + days);
  return dayKey(d);
}

/** `2026-11-04` → `04.11.2026` */
export function formatDay(key: string): string {
  const [y, m, d] = key.split("-");
  return `${d}.${m}.${y}`;
}

export function formatDayLong(key: string): string {
  return parseDayKey(key).toLocaleDateString("tr-TR", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

export function formatTime(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit" });
}

export function formatReceipt(receiptNo: number): string {
  return `#${String(receiptNo).padStart(4, "0")}`;
}

export const PAYMENT_LABEL: Record<PaymentMethod, string> = {
  cash: "Nakit",
  card: "Kart",
};
