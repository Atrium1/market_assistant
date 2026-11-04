import { useCallback, useEffect, useState } from "react";
import { useOwnerSession } from "../auth/OwnerSessionContext";
import { api, errorMessage } from "../lib/api";
import { addDays, dayKey, formatDay, formatDayLong, formatTime } from "../lib/dates";
import { formatTry, parsePriceToCents } from "../lib/money";
import type { DailyReport, DayRow, TopProduct } from "../types";

const RANGE_DAYS = 7;

function TopTable({ title, rows, by }: { title: string; rows: TopProduct[]; by: "qty" | "revenue" }) {
  return (
    <div className="card-panel report-top">
      <h3>{title}</h3>
      {rows.length === 0 ? (
        <p className="muted">—</p>
      ) : (
        <table className="data-table compact">
          <thead>
            <tr>
              <th>Ürün</th>
              <th>Adet</th>
              <th>Tutar</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((p) => (
              <tr key={p.product_id ?? p.barcode}>
                <td>{p.name}</td>
                <td className={by === "qty" ? "strong" : undefined}>{p.qty}</td>
                <td className={by === "revenue" ? "strong" : undefined}>
                  {formatTry(p.revenue_cents)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function CashCount({ expectedCents }: { expectedCents: number }) {
  const [counted, setCounted] = useState("");
  const countedCents = parsePriceToCents(counted);
  const diff = countedCents === null ? null : countedCents - expectedCents;

  return (
    <div className="card-panel cash-count no-print">
      <h3>Kasa sayımı</h3>
      <p className="muted small-note">
        Çekmecedeki nakdi sayıp yazın; beklenen nakit {formatTry(expectedCents)}.
        Gün başında kasada bulunan bozuk parayı düşmeyi unutmayın. Bu bilgi kaydedilmez.
      </p>
      <div className="tender-row">
        <label htmlFor="counted-cash">Sayılan nakit (₺)</label>
        <input
          id="counted-cash"
          value={counted}
          onChange={(e) => setCounted(e.target.value.replace(/[^\d.,]/g, ""))}
          inputMode="decimal"
          autoComplete="off"
        />
        {diff !== null && (
          <span className={`change-value${diff < 0 ? " short" : ""}`}>
            {diff === 0
              ? "Kasa tutuyor"
              : diff > 0
                ? `Fazla: ${formatTry(diff)}`
                : `Eksik: ${formatTry(-diff)}`}
          </span>
        )}
      </div>
    </div>
  );
}

export function DailyReportView() {
  const { requireOwner } = useOwnerSession();
  const today = dayKey();
  const [day, setDay] = useState(today);
  const [report, setReport] = useState<DailyReport | null>(null);
  const [range, setRange] = useState<DayRow[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    setError(null);
    setLoading(true);
    await requireOwner(
      async () => {
        const [r, rows] = await Promise.all([
          api.reportsDaily(day),
          api.reportsRange(addDays(day, -(RANGE_DAYS - 1)), day),
        ]);
        setReport(r);
        setRange(rows);
      },
      (err) => setError(errorMessage(err, "Rapor alınamadı")),
    );
    setLoading(false);
  }, [day, requireOwner]);

  useEffect(() => {
    void load();
  }, [load]);

  const maxHour = Math.max(1, ...(report?.hourly.map((h) => h.total_cents) ?? [0]));
  const activeHours = report?.hourly.filter((h) => h.count > 0) ?? [];
  const firstHour = activeHours.length ? activeHours[0].hour : 8;
  const lastHour = activeHours.length ? activeHours[activeHours.length - 1].hour : 20;
  const shownHours =
    report?.hourly.filter((h) => h.hour >= Math.min(firstHour, 8) && h.hour <= Math.max(lastHour, 20)) ??
    [];

  return (
    <div className="report-view">
      <div className="report-toolbar no-print">
        <button type="button" className="btn" onClick={() => setDay(addDays(day, -1))}>
          ‹ Önceki gün
        </button>
        <input
          type="date"
          value={day}
          max={today}
          onChange={(e) => e.target.value && setDay(e.target.value)}
          aria-label="Gün seç"
        />
        <button
          type="button"
          className="btn"
          onClick={() => setDay(addDays(day, 1))}
          disabled={day >= today}
        >
          Sonraki gün ›
        </button>
        {day !== today && (
          <button type="button" className="btn ghost" onClick={() => setDay(today)}>
            Bugün
          </button>
        )}
        <button type="button" className="btn" onClick={() => void load()} disabled={loading}>
          Yenile
        </button>
        <button type="button" className="btn primary" onClick={() => window.print()}>
          Yazdır
        </button>
      </div>

      <h2 className="report-title">Gün sonu — {formatDayLong(day)}</h2>
      {error && <p className="flash error">{error}</p>}

      {report && (
        <>
          {report.sale_count === 0 && report.voided_count === 0 ? (
            <p className="card-panel muted">Bu gün satış yok.</p>
          ) : (
            <p className="muted">
              İlk satış {formatTime(report.first_sale_at)} · Son satış{" "}
              {formatTime(report.last_sale_at)}
            </p>
          )}

          <div className="report-cards">
            <div className="report-card total">
              <span>Toplam (net)</span>
              <strong>{formatTry(report.net_total_cents)}</strong>
            </div>
            <div className="report-card cash">
              <span>Nakit</span>
              <strong>{formatTry(report.cash_total_cents)}</strong>
            </div>
            <div className="report-card card">
              <span>Kart</span>
              <strong>{formatTry(report.card_total_cents)}</strong>
            </div>
            <div className="report-card">
              <span>Satış sayısı</span>
              <strong>{report.sale_count}</strong>
            </div>
            <div className="report-card">
              <span>Ortalama sepet</span>
              <strong>{formatTry(report.average_basket_cents)}</strong>
            </div>
            <div className="report-card voided">
              <span>İptal edilen</span>
              <strong>
                {report.voided_count} · {formatTry(report.voided_total_cents)}
              </strong>
            </div>
          </div>

          <CashCount key={day} expectedCents={report.cash_total_cents} />

          {activeHours.length > 0 && (
            <div className="card-panel">
              <h3>Saatlere göre</h3>
              <div className="hour-bars">
                {shownHours.map((h) => (
                  <div
                    key={h.hour}
                    className="hour-bar"
                    title={`${String(h.hour).padStart(2, "0")}:00 · ${h.count} satış · ${formatTry(h.total_cents)}`}
                  >
                    <div
                      className="hour-bar-fill"
                      style={{ height: `${(h.total_cents / maxHour) * 100}%` }}
                    />
                    <span>{String(h.hour).padStart(2, "0")}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="report-tops">
            <TopTable title="En çok satan (adet)" rows={report.top_by_qty} by="qty" />
            <TopTable title="En çok kazandıran (tutar)" rows={report.top_by_revenue} by="revenue" />
          </div>
        </>
      )}

      {range.length > 0 && (
        <div className="card-panel">
          <h3>Son {RANGE_DAYS} gün</h3>
          <table className="data-table compact">
            <thead>
              <tr>
                <th>Gün</th>
                <th>Satış</th>
                <th>Nakit</th>
                <th>Kart</th>
                <th>Toplam</th>
                <th>İptal</th>
              </tr>
            </thead>
            <tbody>
              {[...range].reverse().map((r) => (
                <tr
                  key={r.business_day}
                  className={`clickable${r.business_day === day ? " row-highlight" : ""}`}
                  onClick={() => setDay(r.business_day)}
                >
                  <td>{formatDay(r.business_day)}</td>
                  <td>{r.sale_count}</td>
                  <td>{formatTry(r.cash_total_cents)}</td>
                  <td>{formatTry(r.card_total_cents)}</td>
                  <td className="strong">{formatTry(r.net_total_cents)}</td>
                  <td>{r.voided_count || ""}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
