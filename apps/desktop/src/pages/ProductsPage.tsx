import { FormEvent, useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import { formatTry, parsePriceToCents } from "../lib/money";
import { useBarcodeFocus } from "../hooks/useBarcodeFocus";
import type { Product } from "../types";

export function ProductsPage() {
  const [products, setProducts] = useState<Product[]>([]);
  const [barcode, setBarcode] = useState("");
  const [name, setName] = useState("");
  const [price, setPrice] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const { ref: barcodeRef, focusBarcode } = useBarcodeFocus(true);

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
    focusBarcode();
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
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
      focusBarcode();
    }
  }

  function startEdit(product: Product) {
    setEditingId(product.id);
    setBarcode(product.barcode);
    setName(product.name);
    setPrice((product.price_cents / 100).toFixed(2).replace(".", ","));
    setError(null);
    setMessage(null);
    focusBarcode();
  }

  async function deactivate(product: Product) {
    if (!window.confirm(`"${product.name}" pasifleştirilsin mi?`)) {
      focusBarcode();
      return;
    }
    try {
      await api.productsDeactivate(product.id);
      setMessage("Ürün pasifleştirildi");
      if (editingId === product.id) resetForm();
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "İşlem başarısız");
    } finally {
      focusBarcode();
    }
  }

  return (
    <div className="page products-page">
      <header className="page-header">
        <h1>Ürünler</h1>
        <p>Barkod, ad ve fiyat kaydı. Okuyucu barkod alanına yazar.</p>
      </header>

      <form className="product-form card-panel" onSubmit={onSubmit}>
        <div className="form-row">
          <label htmlFor="product-barcode">Barkod</label>
          <input
            id="product-barcode"
            ref={barcodeRef}
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
            {editingId ? "Güncelle" : "Ekle"}
          </button>
          {editingId && (
            <button type="button" className="btn ghost" onClick={resetForm}>
              Vazgeç
            </button>
          )}
        </div>
        {error && <p className="flash error">{error}</p>}
        {message && <p className="flash ok">{message}</p>}
      </form>

      <div className="table-wrap card-panel">
        <table className="data-table">
          <thead>
            <tr>
              <th>Barkod</th>
              <th>Ad</th>
              <th>Fiyat</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {products.length === 0 ? (
              <tr>
                <td colSpan={4} className="muted">
                  Henüz ürün yok. İlk ürünü ekleyin.
                </td>
              </tr>
            ) : (
              products.map((p) => (
                <tr key={p.id}>
                  <td className="mono">{p.barcode}</td>
                  <td>{p.name}</td>
                  <td>{formatTry(p.price_cents)}</td>
                  <td className="row-actions">
                    <button
                      type="button"
                      className="btn small"
                      onClick={() => startEdit(p)}
                    >
                      Düzenle
                    </button>
                    <button
                      type="button"
                      className="btn small danger"
                      onClick={() => void deactivate(p)}
                    >
                      Pasifleştir
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
