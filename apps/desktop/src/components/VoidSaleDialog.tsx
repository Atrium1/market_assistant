import { ask } from "@tauri-apps/plugin-dialog";
import { FormEvent, useEffect, useRef, useState } from "react";
import { useOwnerSession } from "../auth/OwnerSessionContext";
import { api, errorMessage } from "../lib/api";
import { dayKey, formatDay, formatReceipt } from "../lib/dates";
import { formatTry } from "../lib/money";
import type { Sale } from "../types";

const MAX_REASON = 200;

type Props = {
  sale: Sale;
  onClose: () => void;
  onVoided: (sale: Sale) => void;
};

export function VoidSaleDialog({ sale, onClose, onVoided }: Props) {
  const { requireOwner } = useOwnerSession();
  const [reason, setReason] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const reasonRef = useRef<HTMLTextAreaElement | null>(null);
  const notToday = sale.business_day !== dayKey();
  const label = `${formatDay(sale.business_day)} ${formatReceipt(sale.receipt_no)}`;

  useEffect(() => {
    requestAnimationFrame(() => reasonRef.current?.focus());
  }, []);

  async function submit(e: FormEvent) {
    e.preventDefault();
    const trimmed = reason.trim();
    if (!trimmed) {
      setError("İptal nedeni yazın");
      return;
    }
    const confirmed = await ask(
      `${label} numaralı ${formatTry(sale.total_cents)} tutarındaki satış iptal edilsin mi?\n\nSatış silinmez, raporlarda "iptal edilen" olarak görünür.`,
      { title: "Satışı iptal et", kind: "warning", okLabel: "İptal et", cancelLabel: "Vazgeç" },
    );
    if (!confirmed) return;

    setError(null);
    setBusy(true);
    await requireOwner(
      async () => {
        const voided = await api.salesVoid(sale.id, trimmed);
        onVoided(voided);
      },
      (err) => setError(errorMessage(err, "İptal edilemedi")),
    );
    setBusy(false);
  }

  return (
    <div className="modal-backdrop" role="presentation" onClick={busy ? undefined : onClose}>
      <div
        className="modal-card"
        role="dialog"
        aria-modal="true"
        aria-labelledby="void-title"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="void-title">Satışı iptal et</h2>
        <p className="muted">
          {label} · {formatTry(sale.total_cents)}
        </p>
        {notToday && (
          <p className="flash info">
            Bu satış bugüne ait değil. İptal, {formatDay(sale.business_day)} gün sonu
            raporunu değiştirir.
          </p>
        )}
        <form onSubmit={(e) => void submit(e)} className="settings-form">
          <div className="form-row">
            <label htmlFor="void-reason">İptal nedeni</label>
            <textarea
              id="void-reason"
              ref={reasonRef}
              value={reason}
              maxLength={MAX_REASON}
              rows={3}
              onChange={(e) => setReason(e.target.value)}
              placeholder="Örn. yanlış ürün okutuldu, müşteri vazgeçti"
            />
          </div>
          {error && <p className="flash error">{error}</p>}
          <div className="form-actions">
            <button type="button" className="btn ghost" onClick={onClose} disabled={busy}>
              Vazgeç
            </button>
            <button type="submit" className="btn danger" disabled={busy}>
              Satışı iptal et
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
