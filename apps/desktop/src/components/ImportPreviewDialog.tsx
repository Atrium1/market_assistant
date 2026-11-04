import { useEffect, useState } from "react";
import { formatTry } from "../lib/money";
import type { ImportPreview, ImportRow, ImportRowKind } from "../types";

type Props = {
  preview: ImportPreview | null;
  applying: boolean;
  onApply: () => void;
  onClose: () => void;
};

const TABS: Array<{ kind: ImportRowKind; label: string }> = [
  { kind: "add", label: "Yeni" },
  { kind: "update", label: "Güncellenecek" },
  { kind: "unchanged", label: "Değişmeyen" },
  { kind: "invalid", label: "Hatalı" },
];

function price(cents: number | null): string {
  return cents === null ? "—" : formatTry(cents);
}

function RowTable({ kind, rows }: { kind: ImportRowKind; rows: ImportRow[] }) {
  if (rows.length === 0) {
    return <p className="muted">Bu grupta satır yok.</p>;
  }
  return (
    <table className="data-table compact">
      <thead>
        <tr>
          <th>Satır</th>
          <th>Barkod</th>
          <th>Ürün adı</th>
          <th>Fiyat</th>
          {kind === "invalid" && <th>Sorun</th>}
        </tr>
      </thead>
      <tbody>
        {rows.map((r) => (
          <tr key={`${r.row_number}-${r.barcode}`}>
            <td>{r.row_number}</td>
            <td className="mono">{r.barcode || "—"}</td>
            <td>
              {kind === "update" && r.old_name && r.old_name !== r.name ? (
                <>
                  <span className="old-value">{r.old_name}</span> → {r.name}
                </>
              ) : (
                r.name || "—"
              )}
              {r.reactivate && <span className="tag">Silinmişti, geri gelir</span>}
              {r.warning && <div className="row-warning">{r.warning}</div>}
            </td>
            <td>
              {kind === "update" && r.old_price_cents !== r.price_cents ? (
                <>
                  <span className="old-value">{price(r.old_price_cents)}</span> →{" "}
                  {price(r.price_cents)}
                </>
              ) : (
                price(r.price_cents)
              )}
            </td>
            {kind === "invalid" && <td className="row-error">{r.reason}</td>}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function ImportPreviewDialog({ preview, applying, onApply, onClose }: Props) {
  const [tab, setTab] = useState<ImportRowKind>("add");

  useEffect(() => {
    if (!preview) return;
    const first = TABS.find((t) => preview.counts[t.kind] > 0);
    setTab(first?.kind ?? "add");
  }, [preview]);

  if (!preview) return null;

  const applicable = preview.counts.add + preview.counts.update;

  return (
    <div className="modal-backdrop" role="presentation" onClick={applying ? undefined : onClose}>
      <div
        className="modal-card wide"
        role="dialog"
        aria-modal="true"
        aria-labelledby="import-title"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="import-title">İçe aktarma önizlemesi</h2>
        <p className="muted">
          <span className="mono">{preview.file_name}</span> — henüz hiçbir şey
          kaydedilmedi. Kontrol edip onaylayın.
        </p>

        <div className="import-tabs" role="tablist">
          {TABS.map((t) => (
            <button
              key={t.kind}
              type="button"
              role="tab"
              aria-selected={tab === t.kind}
              className={`chip chip-${t.kind}${tab === t.kind ? " active" : ""}`}
              onClick={() => setTab(t.kind)}
            >
              {t.label} <strong>{preview.counts[t.kind]}</strong>
            </button>
          ))}
        </div>

        <div className="import-table table-wrap">
          <RowTable kind={tab} rows={preview[tab]} />
        </div>

        {preview.truncated && (
          <p className="muted small-note">
            Her grupta en fazla 200 satır gösteriliyor; uygulama tüm satırları işler.
          </p>
        )}
        {preview.counts.invalid > 0 && (
          <p className="flash info">
            Hatalı {preview.counts.invalid} satır atlanacak. Dosyayı düzeltip tekrar
            yükleyebilirsiniz.
          </p>
        )}

        <div className="form-actions">
          <button type="button" className="btn ghost" onClick={onClose} disabled={applying}>
            Vazgeç
          </button>
          <button
            type="button"
            className="btn primary"
            onClick={onApply}
            disabled={applying || applicable === 0}
          >
            {applying
              ? "Uygulanıyor…"
              : applicable === 0
                ? "Uygulanacak değişiklik yok"
                : `${applicable} ürünü uygula`}
          </button>
        </div>
      </div>
    </div>
  );
}
