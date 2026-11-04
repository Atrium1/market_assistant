import { useEffect, useState } from "react";
import { useOwnerSession } from "../auth/OwnerSessionContext";
import { api, errorMessage } from "../lib/api";
import { PAYMENT_LABEL, formatDay, formatReceipt } from "../lib/dates";
import { formatDateTime } from "../lib/files";
import { formatTry } from "../lib/money";
import type { Sale, SaleDetail } from "../types";
import { VoidSaleDialog } from "./VoidSaleDialog";

type Props = {
  saleId: string;
  onClose: () => void;
  onVoided: (sale: Sale) => void;
};

export function SaleDetailDialog({ saleId, onClose, onVoided }: Props) {
  const { requireOwner } = useOwnerSession();
  const [detail, setDetail] = useState<SaleDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [voiding, setVoiding] = useState(false);

  useEffect(() => {
    setDetail(null);
    setError(null);
    void requireOwner(
      async () => setDetail(await api.salesGet(saleId)),
      (err) => setError(errorMessage(err, "Satış yüklenemedi")),
    );
  }, [saleId, requireOwner]);

  const sale = detail?.sale;

  return (
    <>
      <div className="modal-backdrop" role="presentation" onClick={onClose}>
        <div
          className="modal-card wide"
          role="dialog"
          aria-modal="true"
          aria-labelledby="sale-title"
          onClick={(e) => e.stopPropagation()}
        >
          <h2 id="sale-title">
            {sale ? `Satış ${formatDay(sale.business_day)} ${formatReceipt(sale.receipt_no)}` : "Satış"}
          </h2>
          {error && <p className="flash error">{error}</p>}
          {sale && detail && (
            <>
              <p className="muted">
                {formatDateTime(sale.paid_at)} · {PAYMENT_LABEL[sale.payment_method]} ·{" "}
                <strong>{formatTry(sale.total_cents)}</strong>
              </p>
              {sale.status === "voided" && (
                <p className="flash error">
                  İptal edildi ({formatDateTime(sale.voided_at)}): {sale.void_reason}
                </p>
              )}
              <div className="table-wrap import-table">
                <table className="data-table compact">
                  <thead>
                    <tr>
                      <th>Barkod</th>
                      <th>Ürün</th>
                      <th>Birim</th>
                      <th>Adet</th>
                      <th>Tutar</th>
                    </tr>
                  </thead>
                  <tbody>
                    {detail.items.map((it) => (
                      <tr key={it.id}>
                        <td className="mono">{it.barcode}</td>
                        <td>{it.name_snapshot}</td>
                        <td>{formatTry(it.unit_price_cents)}</td>
                        <td>{it.qty}</td>
                        <td>{formatTry(it.line_total_cents)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
          <div className="form-actions">
            <button type="button" className="btn ghost" onClick={onClose}>
              Kapat
            </button>
            {sale?.status === "completed" && (
              <button type="button" className="btn danger" onClick={() => setVoiding(true)}>
                İptal et
              </button>
            )}
          </div>
        </div>
      </div>
      {voiding && sale && (
        <VoidSaleDialog
          sale={sale}
          onClose={() => setVoiding(false)}
          onVoided={(voided) => {
            setVoiding(false);
            setDetail((d) => (d ? { ...d, sale: voided } : d));
            onVoided(voided);
          }}
        />
      )}
    </>
  );
}
