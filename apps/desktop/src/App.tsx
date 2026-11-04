import { NavLink, Route, Routes } from "react-router-dom";
import { useOwnerSession } from "./auth/OwnerSessionContext";
import { ProductsPage } from "./pages/ProductsPage";
import { RegisterPage } from "./pages/RegisterPage";
import { ReportsPage } from "./pages/ReportsPage";
import { SettingsPage } from "./pages/SettingsPage";
import "./App.css";

function App() {
  const { loggedIn, usernameDisplay, logout } = useOwnerSession();

  return (
    <div className="app-shell">
      <nav className="app-nav no-print">
        <div className="brand">Market Asistanı</div>
        <div className="nav-links">
          <NavLink to="/" end className={({ isActive }) => (isActive ? "active" : "")}>
            Kasa
          </NavLink>
          <NavLink
            to="/urunler"
            className={({ isActive }) => (isActive ? "active" : "")}
          >
            Ürünler
          </NavLink>
          <NavLink
            to="/raporlar"
            className={({ isActive }) => (isActive ? "active" : "")}
          >
            Raporlar
          </NavLink>
          <NavLink
            to="/ayarlar"
            className={({ isActive }) => (isActive ? "active" : "")}
          >
            Ayarlar
          </NavLink>
        </div>
        <div className="nav-session">
          {loggedIn ? (
            <>
              <span className="session-pill" title="Sahip oturumu">
                {usernameDisplay}
              </span>
              <button
                type="button"
                className="btn small nav-logout"
                onClick={() => void logout()}
              >
                Çıkış
              </button>
            </>
          ) : (
            <span className="session-muted">Çalışan modu</span>
          )}
        </div>
      </nav>
      <main className="app-main">
        <Routes>
          <Route path="/" element={<RegisterPage />} />
          <Route path="/urunler" element={<ProductsPage />} />
          <Route path="/raporlar" element={<ReportsPage />} />
          <Route path="/ayarlar" element={<SettingsPage />} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
