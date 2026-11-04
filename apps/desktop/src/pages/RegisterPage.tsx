import { FormEvent, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { api, cartChangedDetails, errorMessage } from "../lib/api";
import { PAYMENT_LABEL, formatReceipt } from "../lib/dates";
import { formatTry, parsePriceToCents } from "../lib/money";
import { useBarcodeFocus } from "../hooks/useBarcodeFocus";
import type {
  CartChangedDetails,
  CartLine,
  PaymentMethod,
  Product,
  RegisterMode,
  TodaySummary,
} from "../types";

const MODE_LABEL: Record<RegisterMode, string> = {
  sale: "Satış",
  price_check: "Fiyat sor",
  return: "İade",
};

const PAY_KEYS: Record<string, PaymentMethod> = { F8: "cash", F9: "card" };

function cartChangeLines(d: CartChangedDetails): string[] {
  return [
    ...d.changed.map(
      (c) =>
        `${c.name}: ${formatTry(c.old_unit_price_cents)} → ${formatTry(c.new_unit_price_cents)}`,
    ),
    ...d.removed.map((r) => `${r.name ?? "Bilinmeyen ürün"}: artık satışta değil, sepetten çıkarıldı`),
  ];
}

export function RegisterPage() {
  const [mode, setMode] = useState<RegisterMode>("sale");
  const [barcode, setBarcode] = useState("");
  const [cart, setCart] = useState<CartLine[]>([]);
  const [priceHit, setPriceHit] = useState<Product | null>(null);
  const [flash, setFlash] = useState<{ kind: "ok" | "error" | "info"; text: string } | null>(
    null,
  );
  const [cartNotice, setCartNotice] = useState<string[]>([]);
  const [tendered, setTendered] = useState("");
  const [today, setToday] = useState<TodaySummary | null>(null);
  const [busy, setBusy] = useState(false);
  const [removeArmed, setRemoveArmed] = useState(false);
  const { ref: barcodeRef, focusBarcode } = useBarcodeFocus(true);

  const totalCents = useMemo(
    () => cart.reduce((sum, line) => sum + line.unit_price_cents * line.qty, 0),
    [cart],
  );

  const tenderedCents = parsePriceToCents(tendered);
  const changeCents =
    tenderedCents !== null && cart.length > 0 ? tenderedCents - totalCents : null;

  const canArmRemove = cart.length > 0 && !busy && mode === "sale";

  const loadToday = useCallback(async () => {
    try {
      setToday(await api.reportsTodaySummary());
    } catch {
      setToday(null);
    }
  }, []);

  useEffect(() => {
    void loadToday();
    const id = window.setInterval(() => void loadToday(), 60_000);
    return () => window.clearInterval(id);
  }, [loadToday]);

  useEffect(() => {
    if (cart.length === 0) setRemoveArmed(false);
  }, [cart.length]);

  function showFlash(kind: "ok" | "error" | "info", text: string) {
    setFlash({ kind, text });
  }

  function disarmRemove() {
    setRemoveArmed(false);
  }

  function toggleRemoveArm() {
    if (!canArmRemove) return;
    setRemoveArmed((armed) => !armed);
    setFlash(null);
    setPriceHit(null);
    setBarcode("");
    focusBarcode();
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
    disarmRemove();
    setMode(next);
    setPriceHit(null);
    setBarcode("");
    focusBarcode();
  }

  function addToCart(product: Product) {
    setCartNotice([]);
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

  function removeOneByBarcode(code: string) {
    disarmRemove();
    setBarcode("");
    setFlash(null);

    const idx = cart.findIndex((l) => l.barcode.trim() === code);
    if (idx < 0) {
      showFlash("error", "Bu ürün sepette yok");
      focusBarcode();
      return;
    }

    const line = cart[idx];
    if (line.qty <= 1) {
      setCart((prev) => prev.filter((l) => l.product_id !== line.product_id));
      showFlash("ok", `${line.name} sepetten çıkarıldı`);
    } else {
      setCart((prev) =>
        prev.map((l) =>
          l.product_id === line.product_id ? { ...l, qty: l.qty - 1 } : l,
        ),
      );
      showFlash("ok", `${line.name} sepetten 1 adet çıkarıldı`);
    }
    focusBarcode();
  }

  async function handleScan(raw: string) {
    const code = raw.trim();
    if (!code) return;

    if (removeArmed) {
      removeOneByBarcode(code);
      return;
    }

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
      showFlash("error", errorMessage(err, "Ürün bulunamadı"));
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
    disarmRemove();
    setCart([]);
    setPriceHit(null);
    setTendered("");
    setCartNotice([]);
    showFlash("info", "Sepet iptal edildi");
    focusBarcode();
  }

  async function checkout(method: PaymentMethod) {
    if (cart.length === 0 || busy || mode === "return") return;
    disarmRemove();
    setBusy(true);
    setFlash(null);
    setCartNotice([]);
    try {
      const result = await api.salesCheckout(
        cart.map((l) => ({
          product_id: l.product_id,
          qty: l.qty,
          expected_unit_price_cents: l.unit_price_cents,
        })),
        method,
      );
      const { sale } = result;
      let text = `Satış ${formatReceipt(sale.receipt_no)} · ${PAYMENT_LABEL[sale.payment_method]} · ${formatTry(sale.total_cents)}`;
      if (method === "cash" && changeCents !== null && changeCents > 0) {
        text += ` · Para üstü ${formatTry(changeCents)}`;
      }
      setCart([]);
      setPriceHit(null);
      setTendered("");
      showFlash("ok", text);
      void loadToday();
    } catch (err) {
      const changed = cartChangedDetails(err);
      if (changed) {
        setCart(changed.current);
        setCartNotice(cartChangeLines(changed));
        showFlash("error", errorMessage(err, "Sepet güncellendi"));
      } else {
        showFlash("error", errorMessage(err, "Satış kaydedilemedi"));
      }
    } finally {
      setBusy(false);
      focusBarcode();
    }
  }

  const checkoutRef = useRef(checkout);
  checkoutRef.current = checkout;
  const toggleRemoveArmRef = useRef(toggleRemoveArm);
  toggleRemoveArmRef.current = toggleRemoveArm;
  const removeArmedRef = useRef(removeArmed);
  removeArmedRef.current = removeArmed;

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "F4") {
        e.preventDefault();
        toggleRemoveArmRef.current();
        return;
      }
      if (e.key === "Escape" && removeArmedRef.current) {
        e.preventDefault();
        setRemoveArmed(false);
        setFlash({ kind: "info", text: "Ürün iptal iptal edildi" });
        focusBarcode();
        return;
      }
      const method = PAY_KEYS[e.key];
      if (!method) return;
      e.preventDefault();
      void checkoutRef.current(method);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [focusBarcode]);

  const canPay = cart.length > 0 && !busy && mode !== "return";

  return (
    <div
      className={`page register-page mode-${mode}${removeArmed ? " remove-armed" : ""}`}
    >
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
        {today && (
          <div className="today-chip" title="Bugünkü satışlar (iptaller hariç)">
            Bugün: <strong>{formatTry(today.net_total_cents)}</strong> · {today.sale_count} satış
          </div>
        )}
        <div className="total-panel" aria-live="polite">
          <span className="total-label">Toplam</span>
          <span className="total-value">{formatTry(totalCents)}</span>
        </div>
      </header>

      <div
        className={`mode-banner ${removeArmed ? "mode-remove" : `mode-${mode}`}`}
      >
        {removeArmed
          ? "Ürün iptal — iptal edilecek ürünü okutun (vazgeçmek için Esc)"
          : mode === "sale"
            ? "Satış modu — okutulan ürün sepete eklenir"
            : mode === "price_check"
              ? "Fiyat sor — sepete eklenmez; isterseniz Sepete ekle kullanın"
              : "İade modu yakında — iade akışı henüz aktif değil"}
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
              placeholder={
                removeArmed ? "İptal edilecek ürünü okutun…" : "Barkodu okutun…"
              }
            />
          </form>

          {flash && (
            <p className={`flash ${flash.kind}`} aria-live="polite">
              {flash.text}
            </p>
          )}
          {cartNotice.length > 0 && (
            <ul className="cart-notice">
              {cartNotice.map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          )}
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

          {mode === "sale" && cart.length > 0 && (
            <div className="tender-row">
              <label htmlFor="tendered">Alınan (₺)</label>
              <input
                id="tendered"
                value={tendered}
                onChange={(e) => setTendered(e.target.value.replace(/[^\d.,]/g, ""))}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    focusBarcode();
                  }
                  // Escape while remove-armed is handled by the window listener
                  if (e.key === "Escape" && !removeArmed) {
                    e.preventDefault();
                    focusBarcode();
                  }
                }}
                inputMode="decimal"
                autoComplete="off"
                placeholder="Nakit için isteğe bağlı"
              />
              <span
                className={`change-value${changeCents !== null && changeCents < 0 ? " short" : ""}`}
              >
                {changeCents === null
                  ? ""
                  : changeCents >= 0
                    ? `Para üstü: ${formatTry(changeCents)}`
                    : `Eksik: ${formatTry(-changeCents)}`}
              </span>
            </div>
          )}

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
              className={`btn remove-arm${removeArmed ? " active" : ""}`}
              onClick={toggleRemoveArm}
              disabled={!canArmRemove}
              aria-pressed={removeArmed}
            >
              Ürün iptal <kbd>F4</kbd>
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
              className="btn pay pay-cash"
              onClick={() => void checkout("cash")}
              disabled={!canPay}
            >
              Nakit <kbd>F8</kbd>
            </button>
            <button
              type="button"
              className="btn pay pay-card"
              onClick={() => void checkout("card")}
              disabled={!canPay}
            >
              Kart <kbd>F9</kbd>
            </button>
          </div>
        </section>
      </div>
    </div>
  );
}
