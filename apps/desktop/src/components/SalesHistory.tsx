import { useCallback, useEffect, useState } from "react";
import { useOwnerSession } from "../auth/OwnerSessionContext";
import { api, errorMessage } from "../lib/api";
import { PAYMENT_LABEL, addDays, dayKey, formatDay, formatReceipt, formatTime } from "../lib/dates";
import { formatTry } from "../lib/money";
import type { PaymentMethod, Sale, SaleListItem, SalesListQuery } from "../types";
import { SaleDetailDialog } from "./SaleDetailDialog";

export function SalesHistory() {
  const { requireOwner } = useOwnerSession();
  const today = dayKey();
  const [from, setFrom] = useState(addDays(today, -6));
  const [to, setTo] = useState(today);
  const [payment, setPayment] = useState<"" | PaymentMethod>("");
  const [showVoided, setShowVoided] = useState(true);
  const [items, setItems] = useState<SaleListItem[]>([]);
  const [cursor, setCursor] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const fetchPage = useCallback(
    async (after: string | null) => {
      const query: SalesListQuery = {
        from,
        to,
        payment_method: payment || undefined,
        status: showVoided ? undefined : "completed",
        cursor: after ?? undefined,
      };
      setError(null);
      setLoading(true);
      await requireOwner(
        async () => {
          const page = await api.salesList(query);
          setItems((prev) => (after ? [...prev, ...page.items] : page.items));
          setCursor(page.next_cursor);
        },
        (err) => setError(errorMessage(err, "Satışlar alınamadı")),
      );
      setLoading(false);
    },
    [from, to, payment, showVoided, requireOwner],
  );

  useEffect(() => {
    void fetchPage(null);
  }, [fetchPage]);

  function onVoided(sale: Sale) {
    setItems((prev) =>
      prev.map((s) =>
        s.id === sale.id ? { ...s, status: sale.status, void_reason: sale.void_reason } : s,
      ),
    );
  }

  return (
    <div className="report-view">
      <div className="report-toolbar">
        <label>
          Başlangıç
          <input
            type="date"
            value={from}
            max={to}
            onChange={(e) => e.target.value && setFrom(e.target.value)}
          />
        </label>
        <label>
          Bitiş
          <input
            type="date"
            value={to}
            min={from}
            max={today}
            onChange={(e) => e.target.value && setTo(e.target.value)}
          />
        </label>
        <label>
          Ödeme
          <select value={payment} onChange={(e) => setPayment(e.target.value as "" | PaymentMethod)}>
            <option value="">Tümü</option>
            <option value="cash">Nakit</option>
            <option value="card">Kart</option>
          </select>
        </label>
        <label className="checkbox">
          <input
            type="checkbox"
            checked={showVoided}
            onChange={(e) => setShowVoided(e.target.checked)}
          />
          İptalleri göster
        </label>
      </div>

      {error && <p className="flash error">{error}</p>}

      <div className="card-panel table-wrap">
        <table className="data-table">
          <thead>
            <tr>
              <th>Fiş</th>
              <th>Saat</th>
              <th>Ödeme</th>
              <th>Adet</th>
              <th>Tutar</th>
              <th>Durum</th>
            </tr>
          </thead>
          <tbody>
            {items.length === 0 ? (
              <tr>
                <td colSpan={6} className="muted">
                  {loading ? "Yükleniyor…" : "Bu aralıkta satış yok."}
                </td>
              </tr>
            ) : (
              items.map((s) => (
                <tr
                  key={s.id}
                  className={`clickable${s.status === "voided" ? " voided-row" : ""}`}
                  onClick={() => setSelectedId(s.id)}
                >
                  <td>
                    {formatDay(s.business_day)} <span className="mono">{formatReceipt(s.receipt_no)}</span>
                  </td>
                  <td>{formatTime(s.paid_at)}</td>
                  <td>{PAYMENT_LABEL[s.payment_method]}</td>
                  <td>{s.item_count}</td>
                  <td className="amount">{formatTry(s.total_cents)}</td>
                  <td>
                    {s.status === "voided" ? (
                      <span className="void-tag" title={s.void_reason ?? ""}>
                        İptal: {s.void_reason}
                      </span>
                    ) : (
                      "Tamamlandı"
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {cursor && (
        <button
          type="button"
          className="btn"
          disabled={loading}
          onClick={() => void fetchPage(cursor)}
        >
          {loading ? "Yükleniyor…" : "Daha fazla"}
        </button>
      )}

      {selectedId && (
        <SaleDetailDialog
          saleId={selectedId}
          onClose={() => setSelectedId(null)}
          onVoided={onVoided}
        />
      )}
    </div>
  );
}
