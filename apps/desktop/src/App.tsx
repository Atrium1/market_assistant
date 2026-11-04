import { NavLink, Route, Routes } from "react-router-dom";
import { ProductsPage } from "./pages/ProductsPage";
import { RegisterPage } from "./pages/RegisterPage";
import "./App.css";

function App() {
  return (
    <div className="app-shell">
      <nav className="app-nav">
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
        </div>
      </nav>
      <main className="app-main">
        <Routes>
          <Route path="/" element={<RegisterPage />} />
          <Route path="/urunler" element={<ProductsPage />} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
