import { ask } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import { formatBytes, formatDateTime, pickFile } from "../lib/files";
import type { BackupInfo } from "../types";

const KIND_LABEL: Array<[string, string]> = [
  ["auto_", "Otomatik"],
  ["pre_restore_", "Geri yükleme öncesi"],
  ["pre_import_", "İçe aktarma öncesi"],
];

function backupKind(name: string): string {
  return KIND_LABEL.find(([prefix]) => name.startsWith(prefix))?.[1] ?? "Diğer";
}

export function BackupCard() {
  const [info, setInfo] = useState<BackupInfo | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      setInfo(await api.backupInfo());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Yedek bilgisi alınamadı");
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function run(action: () => Promise<void>) {
    setError(null);
    setMessage(null);
    setBusy(true);
    try {
      await action();
    } catch (e) {
      setError(e instanceof Error ? e.message : "İşlem başarısız");
    } finally {
      setBusy(false);
    }
  }

  const createBackup = () =>
    run(async () => {
      const dest = await pickFile.backupSaveTarget();
      if (!dest) return;
      await api.backupCreate(dest);
      setMessage(`Yedek kaydedildi: ${dest}`);
    });

  const restoreBackup = () =>
    run(async () => {
      const src = await pickFile.backupSource();
      if (!src) return;
      const summary = await api.backupInspect(src);
      const confirmed = await ask(
        [
          `Yedekte ${summary.product_count} ürün ve ${summary.sale_count} satış var.`,
          `Son satış: ${formatDateTime(summary.last_sale_at)}`,
          "",
          "Şu anki tüm ürünler, satışlar ve ayarlar bu yedekle DEĞİŞTİRİLECEK.",
          "Mevcut veriler önce otomatik olarak yedeklenir.",
          "",
          "Devam edilsin mi?",
        ].join("\n"),
        { title: "Yedekten geri yükle", kind: "warning", okLabel: "Geri yükle", cancelLabel: "Vazgeç" },
      );
      if (!confirmed) return;
      await api.backupRestore(src);
      window.location.reload();
    });

  async function copyPath() {
    if (!info) return;
    try {
      await navigator.clipboard.writeText(info.db_path);
      setMessage("Konum kopyalandı");
    } catch {
      setError("Kopyalanamadı; konumu elle seçip kopyalayın");
    }
  }

  return (
    <section className="card-panel settings-form backup-card">
      <h2 className="panel-title">Yedekleme</h2>
      <p className="muted">
        Ürün listesi ve satışlar bu bilgisayarda tek bir dosyada tutulur. Düzenli
        olarak USB belleğe veya başka bir diske yedek alın.
      </p>

      <div className="form-actions">
        <button type="button" className="btn primary" disabled={busy} onClick={() => void createBackup()}>
          Yedekle
        </button>
        <button type="button" className="btn danger" disabled={busy} onClick={() => void restoreBackup()}>
          Yedekten geri yükle
        </button>
      </div>

      {error && <p className="flash error">{error}</p>}
      {message && <p className="flash ok">{message}</p>}

      <div className="form-row">
        <label htmlFor="db-path">Veritabanı konumu</label>
        <div className="path-row">
          <input id="db-path" readOnly value={info?.db_path ?? ""} className="mono" />
          <button type="button" className="btn small" onClick={() => void copyPath()}>
            Kopyala
          </button>
        </div>
      </div>

      <div className="auto-backups">
        <h3>Otomatik yedekler</h3>
        <p className="muted small-note">
          Uygulama her gün ilk açılışta bir kopya alır (son 7 gün saklanır).
          Bunlar aynı diskte durduğu için yanlışlıkla silmeye karşı korur;
          bilgisayar arızasına karşı USB yedeği gerekir.
        </p>
        {info && info.recent.length > 0 ? (
          <table className="data-table compact">
            <thead>
              <tr>
                <th>Dosya</th>
                <th>Tür</th>
                <th>Tarih</th>
                <th>Boyut</th>
              </tr>
            </thead>
            <tbody>
              {info.recent.map((f) => (
                <tr key={f.name}>
                  <td className="mono">{f.name}</td>
                  <td>{backupKind(f.name)}</td>
                  <td>{formatDateTime(f.modified_at)}</td>
                  <td>{formatBytes(f.size_bytes)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p className="muted">Henüz otomatik yedek yok.</p>
        )}
        {info && (
          <p className="muted small-note">
            Klasör: <span className="mono">{info.backup_dir}</span>
          </p>
        )}
      </div>
    </section>
  );
}
