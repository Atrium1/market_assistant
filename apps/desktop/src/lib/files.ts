import { open, save } from "@tauri-apps/plugin-dialog";

export function todayStamp(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

const DB_FILTER = [{ name: "Market Asistanı yedeği", extensions: ["db"] }];
const XLSX_FILTER = [{ name: "Excel", extensions: ["xlsx"] }];
const IMPORT_FILTER = [
  { name: "Excel veya CSV", extensions: ["xlsx", "xls", "csv"] },
];

async function pickOne(options: Parameters<typeof open>[0]): Promise<string | null> {
  const result = await open({ ...options, multiple: false, directory: false });
  return typeof result === "string" ? result : null;
}

export const pickFile = {
  backupSaveTarget: () =>
    save({
      title: "Yedeği kaydet",
      defaultPath: `market_yedek_${todayStamp()}.db`,
      filters: DB_FILTER,
    }),
  backupSource: () =>
    pickOne({ title: "Geri yüklenecek yedeği seçin", filters: DB_FILTER }),
  importSource: () =>
    pickOne({ title: "İçe aktarılacak dosyayı seçin", filters: IMPORT_FILTER }),
  exportTarget: () =>
    save({
      title: "Ürün listesini kaydet",
      defaultPath: `urunler_${todayStamp()}.xlsx`,
      filters: XLSX_FILTER,
    }),
  templateTarget: () =>
    save({
      title: "Şablonu kaydet",
      defaultPath: "urun_sablonu.xlsx",
      filters: XLSX_FILTER,
    }),
};

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function formatDateTime(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString("tr-TR", { dateStyle: "medium", timeStyle: "short" });
}
