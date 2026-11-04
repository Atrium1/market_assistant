import { useState } from "react";
import { useOwnerSession } from "../auth/OwnerSessionContext";
import { DailyReportView } from "../components/DailyReportView";
import { SalesHistory } from "../components/SalesHistory";

type Tab = "daily" | "history";

export function ReportsPage() {
  const { loggedIn, openLogin } = useOwnerSession();
  const [tab, setTab] = useState<Tab>("daily");

  if (!loggedIn) {
    return (
      <div className="page settings-page">
        <header className="page-header">
          <h1>Raporlar</h1>
          <p>Gün sonu raporu ve satış geçmişi için sahip girişi gerekir.</p>
        </header>
        <div className="card-panel settings-locked">
          <p>Raporları görmek için giriş yapın.</p>
          <button type="button" className="btn primary" onClick={() => openLogin()}>
            Giriş yap
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="page reports-page">
      <header className="page-header products-header no-print">
        <div>
          <h1>Raporlar</h1>
          <p>Gün sonu kasa raporu, satış geçmişi ve satış iptali.</p>
        </div>
        <div className="import-tabs" role="tablist">
          <button
            type="button"
            role="tab"
            aria-selected={tab === "daily"}
            className={`chip${tab === "daily" ? " active" : ""}`}
            onClick={() => setTab("daily")}
          >
            Gün sonu
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={tab === "history"}
            className={`chip${tab === "history" ? " active" : ""}`}
            onClick={() => setTab("history")}
          >
            Satış geçmişi
          </button>
        </div>
      </header>
      {tab === "daily" ? <DailyReportView /> : <SalesHistory />}
    </div>
  );
}
