import { FormEvent, useCallback, useEffect, useRef, useState } from "react";
import { useOwnerSession } from "../auth/OwnerSessionContext";
import { useBarcodeFocus } from "../hooks/useBarcodeFocus";
import { api } from "../lib/api";
import { formatTry, parsePriceToCents } from "../lib/money";
import type { Product } from "../types";

export function ProductsPage() {
  const { loggedIn, openLogin, logout } = useOwnerSession();
  const [products, setProducts] = useState<Product[]>([]);
  const [barcode, setBarcode] = useState("");
  const [name, setName] = useState("");
  const [price, setPrice] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [scanBarcode, setScanBarcode] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const formBarcodeRef = useRef<HTMLInputElement | null>(null);
  const { ref: scanRef, focusBarcode: focusScan } = useBarcodeFocus(loggedIn);

  function focusFormBarcode() {
    requestAnimationFrame(() => {
      formBarcodeRef.current?.focus();
      formBarcodeRef.current?.select();
    });
  }

  const load = useCallback(async () => {
    try {
      const list = await api.productsList();
      setProducts(list);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Liste alınamadı");
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  function resetForm() {
    setBarcode("");
    setName("");
    setPrice("");
    setEditingId(null);
    focusFormBarcode();
  }

  function fillFromProduct(product: Product) {
    setEditingId(product.id);
    setBarcode(product.barcode);
    setName(product.name);
    setPrice((product.price_cents / 100).toFixed(2).replace(".", ","));
    setError(null);
    setMessage(null);
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    if (!loggedIn) return;
    setError(null);
    setMessage(null);

    const price_cents = parsePriceToCents(price);
    if (price_cents === null) {
      setError("Geçerli bir fiyat girin (ör. 12,50)");
      return;
    }
    if (!barcode.trim() || !name.trim()) {
      setError("Barkod ve ürün adı zorunludur");
      return;
    }

    setLoading(true);
    try {
      if (editingId) {
        await api.productsUpdate({
          id: editingId,
          barcode: barcode.trim(),
          name: name.trim(),
          price_cents,
        });
        setMessage("Ürün güncellendi");
      } else {
        await api.productsCreate({
          barcode: barcode.trim(),
          name: name.trim(),
          price_cents,
        });
        setMessage("Ürün eklendi");
      }
      resetForm();
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Kayıt başarısız");
    } finally {
      setLoading(false);
      focusFormBarcode();
    }
  }

  async function onScanSubmit(e: FormEvent) {
    e.preventDefault();
    if (!loggedIn) return;
    const code = scanBarcode.trim();
    if (!code) return;
    setScanBarcode("");
    setError(null);
    try {
      const product = await api.productsGetByBarcode(code);
      fillFromProduct(product);
      setMessage(`Okutuldu: ${product.name} — yeni fiyatı girip Kaydet’e basın`);
      focusFormBarcode();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Ürün bulunamadı");
      focusScan();
    }
  }

  async function removeProduct(product: Product) {
    if (!loggedIn) return;
    if (!window.confirm(`"${product.name}" silinsin mi? (listeden kaldırılır)`)) {
      focusFormBarcode();
      return;
    }
    try {
      await api.productsDeactivate(product.id);
      setMessage("Ürün silindi");
      if (editingId === product.id) resetForm();
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "İşlem başarısız");
    } finally {
      focusFormBarcode();
    }
  }

  async function exitEditMode() {
    await logout();
    resetForm();
    setScanBarcode("");
    setMessage(null);
    setError(null);
  }

  return (
    <div className={`page products-page${loggedIn ? " is-editing" : ""}`}>
      <header className="page-header products-header">
        <div>
          <h1>Ürünler</h1>
          <p>
            {loggedIn
              ? "Düzenleme modu açık — ekle, sil, fiyat güncelle."
              : "Liste herkese açık. Değişiklik için Düzenleme modu gerekir."}
          </p>
        </div>
        <div className="products-header-actions">
          {loggedIn ? (
            <button type="button" className="btn" onClick={() => void exitEditMode()}>
              Düzenlemeden çık
            </button>
          ) : (
            <button
              type="button"
              className="btn primary"
              onClick={() => openLogin()}
            >
              Düzenleme modu
            </button>
          )}
        </div>
      </header>

      {loggedIn && (
        <>
          <form
            className="scan-price-panel card-panel"
            onSubmit={(e) => void onScanSubmit(e)}
          >
            <h2 className="panel-title">Barkod ile bul / fiyat güncelle</h2>
            <p className="muted">
              Barkodu okutun; ürün forma dolar. Yeni fiyatı yazıp Kaydet’e basın.
            </p>
            <div className="form-row">
              <label htmlFor="scan-product-barcode">Barkod okut</label>
              <input
                id="scan-product-barcode"
                ref={scanRef}
                value={scanBarcode}
                onChange={(e) => setScanBarcode(e.target.value)}
                autoComplete="off"
                placeholder="Barkodu okutun…"
              />
            </div>
          </form>

          <form className="product-form card-panel" onSubmit={(e) => void onSubmit(e)}>
            <h2 className="panel-title">
              {editingId ? "Ürünü güncelle" : "Yeni ürün"}
            </h2>
            <div className="form-row">
              <label htmlFor="product-barcode">Barkod</label>
              <input
                id="product-barcode"
                ref={formBarcodeRef}
                value={barcode}
                onChange={(e) => setBarcode(e.target.value)}
                autoComplete="off"
                placeholder="Barkodu okutun veya yazın"
              />
            </div>
            <div className="form-row">
              <label htmlFor="product-name">Ürün adı</label>
              <input
                id="product-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                autoComplete="off"
                placeholder="Örn. Su 0.5L"
              />
            </div>
            <div className="form-row">
              <label htmlFor="product-price">Fiyat (₺)</label>
              <input
                id="product-price"
                value={price}
                onChange={(e) => setPrice(e.target.value)}
                autoComplete="off"
                placeholder="12,50"
                inputMode="decimal"
              />
            </div>
            <div className="form-actions">
              <button type="submit" className="btn primary" disabled={loading}>
                {editingId ? "Kaydet" : "Ekle"}
              </button>
              {editingId && (
                <button type="button" className="btn ghost" onClick={resetForm}>
                  Vazgeç
                </button>
              )}
            </div>
          </form>
        </>
      )}

      {(error || message) && (
        <div className="products-flash">
          {error && <p className="flash error">{error}</p>}
          {message && <p className="flash ok">{message}</p>}
        </div>
      )}

      <div className="table-wrap card-panel">
        <table className="data-table">
          <thead>
            <tr>
              <th>Barkod</th>
              <th>Ad</th>
              <th>Fiyat</th>
              {loggedIn && <th></th>}
            </tr>
          </thead>
          <tbody>
            {products.length === 0 ? (
              <tr>
                <td colSpan={loggedIn ? 4 : 3} className="muted">
                  Henüz ürün yok
                  {loggedIn
                    ? ". Düzenleme modunda ilk ürünü ekleyin."
                    : ". Sahip düzenleme modunda ekleyebilir."}
                </td>
              </tr>
            ) : (
              products.map((p) => (
                <tr key={p.id}>
                  <td className="mono">{p.barcode}</td>
                  <td>{p.name}</td>
                  <td>{formatTry(p.price_cents)}</td>
                  {loggedIn && (
                    <td className="row-actions">
                      <button
                        type="button"
                        className="btn small"
                        onClick={() => {
                          fillFromProduct(p);
                          focusFormBarcode();
                        }}
                      >
                        Düzenle
                      </button>
                      <button
                        type="button"
                        className="btn small danger"
                        onClick={() => void removeProduct(p)}
                      >
                        Sil
                      </button>
                    </td>
                  )}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
