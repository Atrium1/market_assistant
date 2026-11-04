import { FormEvent, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { api } from "../lib/api";
import { formatTry } from "../lib/money";
import { useBarcodeFocus } from "../hooks/useBarcodeFocus";
import type { CartLine, Product, RegisterMode } from "../types";

const MODE_LABEL: Record<RegisterMode, string> = {
  sale: "Satış",
  price_check: "Fiyat sor",
  return: "İade",
};

export function RegisterPage() {
  const [mode, setMode] = useState<RegisterMode>("sale");
  const [barcode, setBarcode] = useState("");
  const [cart, setCart] = useState<CartLine[]>([]);
  const [priceHit, setPriceHit] = useState<Product | null>(null);
  const [flash, setFlash] = useState<{ kind: "ok" | "error" | "info"; text: string } | null>(
    null,
  );
  const [busy, setBusy] = useState(false);
  const { ref: barcodeRef, focusBarcode } = useBarcodeFocus(true);

  const totalCents = useMemo(
    () => cart.reduce((sum, line) => sum + line.unit_price_cents * line.qty, 0),
    [cart],
  );

  function showFlash(kind: "ok" | "error" | "info", text: string) {
    setFlash({ kind, text });
  }

  function trySetMode(next: RegisterMode) {
    if (next === "return" && cart.length > 0) {
      showFlash(
        "error",
        "Önce sepeti tamamla veya iptal et",
      );
      focusBarcode();
      return;
    }
    setMode(next);
    setPriceHit(null);
    setBarcode("");
    focusBarcode();
  }

  function addToCart(product: Product) {
    setCart((prev) => {
      const idx = prev.findIndex((l) => l.product_id === product.id);
      if (idx >= 0) {
        const copy = [...prev];
        copy[idx] = { ...copy[idx], qty: copy[idx].qty + 1 };
        return copy;
      }
      return [
        ...prev,
        {
          product_id: product.id,
          barcode: product.barcode,
          name: product.name,
          unit_price_cents: product.price_cents,
          qty: 1,
        },
      ];
    });
  }

  async function handleScan(raw: string) {
    const code = raw.trim();
    if (!code) return;

    setBarcode("");
    setFlash(null);

    if (mode === "return") {
      showFlash("info", "İade yakında — şimdilik Satış veya Fiyat sor kullanın");
      focusBarcode();
      return;
    }

    setBusy(true);
    try {
      const product = await api.productsGetByBarcode(code);
      if (mode === "sale") {
        addToCart(product);
        setPriceHit(null);
      } else {
        setPriceHit(product);
      }
    } catch (err) {
      setPriceHit(null);
      const msg = err instanceof Error ? err.message : "Ürün bulunamadı";
      showFlash("error", msg);
    } finally {
      setBusy(false);
      focusBarcode();
    }
  }

  function onBarcodeSubmit(e: FormEvent) {
    e.preventDefault();
    void handleScan(barcode);
  }

  function changeQty(productId: string, qty: number) {
    setCart((prev) => {
      if (qty <= 0) return prev.filter((l) => l.product_id !== productId);
      return prev.map((l) =>
        l.product_id === productId ? { ...l, qty } : l,
      );
    });
    focusBarcode();
  }

  function removeLine(productId: string) {
    setCart((prev) => prev.filter((l) => l.product_id !== productId));
    focusBarcode();
  }

  function removeLast() {
    setCart((prev) => prev.slice(0, -1));
    focusBarcode();
  }

  function voidCart() {
    if (cart.length === 0) {
      focusBarcode();
      return;
    }
    if (!window.confirm("Sepet tamamen iptal edilsin mi?")) {
      focusBarcode();
      return;
    }
    setCart([]);
    setPriceHit(null);
    showFlash("info", "Sepet iptal edildi");
    focusBarcode();
  }

  async function checkout() {
    if (cart.length === 0) return;
    setBusy(true);
    setFlash(null);
    try {
      const result = await api.salesCheckout(
        cart.map((l) => ({
          product_id: l.product_id,
          barcode: l.barcode,
          name: l.name,
          unit_price_cents: l.unit_price_cents,
          qty: l.qty,
        })),
      );
      setCart([]);
      setPriceHit(null);
      showFlash(
        "ok",
        `Satış tamamlandı — ${formatTry(result.sale.total_cents)}`,
      );
    } catch (err) {
      showFlash(
        "error",
        err instanceof Error ? err.message : "Satış kaydedilemedi",
      );
    } finally {
      setBusy(false);
      focusBarcode();
    }
  }

  return (
    <div className={`page register-page mode-${mode}`}>
      <header className="register-top">
        <div className="mode-switch" role="tablist" aria-label="Kasa modu">
          {(Object.keys(MODE_LABEL) as RegisterMode[]).map((m) => (
            <button
              key={m}
              type="button"
              role="tab"
              aria-selected={mode === m}
              className={`mode-btn mode-${m} ${mode === m ? "active" : ""}`}
              onClick={() => trySetMode(m)}
            >
              {MODE_LABEL[m]}
            </button>
          ))}
        </div>
        <div className="total-panel" aria-live="polite">
          <span className="total-label">Toplam</span>
          <span className="total-value">{formatTry(totalCents)}</span>
        </div>
      </header>

      <div className={`mode-banner mode-${mode}`}>
        {mode === "sale" && "Satış modu — okutulan ürün sepete eklenir"}
        {mode === "price_check" &&
          "Fiyat sor — sepete eklenmez; isterseniz Sepete ekle kullanın"}
        {mode === "return" && "İade modu yakında — iade akışı henüz aktif değil"}
      </div>

      <div className="register-body">
        <section className="cart-panel card-panel">
          <form className="scan-form" onSubmit={onBarcodeSubmit}>
            <label htmlFor="scan-barcode">Barkod</label>
            <input
              id="scan-barcode"
              ref={barcodeRef}
              value={barcode}
              onChange={(e) => setBarcode(e.target.value)}
              disabled={busy}
              autoComplete="off"
              placeholder="Barkodu okutun…"
            />
          </form>

          {flash && <p className={`flash ${flash.kind}`}>{flash.text}</p>}
          {flash?.kind === "error" &&
            flash.text.includes("bulunamadı") && (
              <p className="hint">
                <Link to="/urunler" onClick={() => focusBarcode()}>
                  Ürünlere git ve kaydet →
                </Link>
              </p>
            )}

          {mode === "price_check" && priceHit && (
            <div className="price-hit">
              <div className="price-hit-name">{priceHit.name}</div>
              <div className="price-hit-price">
                {formatTry(priceHit.price_cents)}
              </div>
              <button
                type="button"
                className="btn primary"
                onClick={() => {
                  addToCart(priceHit);
                  setMode("sale");
                  setPriceHit(null);
                  showFlash("ok", "Sepete eklendi — Satış moduna geçildi");
                  focusBarcode();
                }}
              >
                Sepete ekle
              </button>
            </div>
          )}

          <ul className="cart-lines">
            {cart.length === 0 ? (
              <li className="muted empty-cart">Sepet boş</li>
            ) : (
              cart.map((line) => (
                <li key={line.product_id} className="cart-line">
                  <div className="cart-line-main">
                    <strong>{line.name}</strong>
                    <span className="mono muted">{line.barcode}</span>
                  </div>
                  <div className="cart-line-qty">
                    <button
                      type="button"
                      className="btn small"
                      onClick={() => changeQty(line.product_id, line.qty - 1)}
                    >
                      −
                    </button>
                    <span>{line.qty}</span>
                    <button
                      type="button"
                      className="btn small"
                      onClick={() => changeQty(line.product_id, line.qty + 1)}
                    >
                      +
                    </button>
                  </div>
                  <div className="cart-line-total">
                    {formatTry(line.unit_price_cents * line.qty)}
                  </div>
                  <button
                    type="button"
                    className="btn small danger"
                    onClick={() => removeLine(line.product_id)}
                  >
                    Sil
                  </button>
                </li>
              ))
            )}
          </ul>

          <div className="cart-actions">
            <button
              type="button"
              className="btn"
              onClick={removeLast}
              disabled={cart.length === 0 || busy}
            >
              Sonunu sil
            </button>
            <button
              type="button"
              className="btn danger"
              onClick={voidCart}
              disabled={cart.length === 0 || busy}
            >
              Sepeti iptal
            </button>
            <button
              type="button"
              className="btn primary checkout"
              onClick={() => void checkout()}
              disabled={cart.length === 0 || busy || mode === "return"}
            >
              Satışı tamamla
            </button>
          </div>
        </section>
      </div>
    </div>
  );
}
