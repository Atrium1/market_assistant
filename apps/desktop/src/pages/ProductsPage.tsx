import {
  FormEvent,
  KeyboardEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useOwnerSession } from "../auth/OwnerSessionContext";
import { ImportPreviewDialog } from "../components/ImportPreviewDialog";
import { useBarcodeFocus } from "../hooks/useBarcodeFocus";
import { api } from "../lib/api";
import { pickFile } from "../lib/files";
import { formatTry, parsePriceToCents } from "../lib/money";
import { normalizeTr, possessiveSuffixTr } from "../lib/text";
import type { ImportPreview, Product } from "../types";

const HIGHLIGHT_MS = 2000;

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
  const [search, setSearch] = useState("");
  const [searchMiss, setSearchMiss] = useState(false);
  const [highlightId, setHighlightId] = useState<string | null>(null);
  const [importPath, setImportPath] = useState<string | null>(null);
  const [importPreview, setImportPreview] = useState<ImportPreview | null>(null);
  const [bulkBusy, setBulkBusy] = useState(false);
  const formBarcodeRef = useRef<HTMLInputElement | null>(null);
  const tableRef = useRef<HTMLTableSectionElement | null>(null);
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

  const searchIndex = useMemo(
    () => products.map((p) => ({ product: p, name: normalizeTr(p.name) })),
    [products],
  );

  const filtered = useMemo(() => {
    const raw = search.trim();
    if (!raw) return products;
    const q = normalizeTr(raw);
    return searchIndex
      .filter((e) => e.name.includes(q) || e.product.barcode.includes(raw))
      .map((e) => e.product);
  }, [search, products, searchIndex]);

  useEffect(() => {
    if (!highlightId) return;
    tableRef.current
      ?.querySelector(`[data-product-id="${highlightId}"]`)
      ?.scrollIntoView({ block: "center", behavior: "smooth" });
    const timer = window.setTimeout(() => setHighlightId(null), HIGHLIGHT_MS);
    return () => window.clearTimeout(timer);
  }, [highlightId]);

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

  function onSearchKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Escape") {
      setSearch("");
      setSearchMiss(false);
      return;
    }
    if (e.key !== "Enter") return;
    e.preventDefault();
    const raw = search.trim();
    if (!raw) return;
    const exact = products.find((p) => p.barcode === raw);
    if (exact) {
      setSearch("");
      setSearchMiss(false);
      setHighlightId(exact.id);
      if (loggedIn) {
        fillFromProduct(exact);
        setMessage(`Bulundu: ${exact.name} — formda düzenleyebilirsiniz`);
      }
      return;
    }
    setSearchMiss(filtered.length === 0);
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

  async function runBulk(action: () => Promise<void>) {
    setError(null);
    setMessage(null);
    setBulkBusy(true);
    try {
      await action();
    } catch (err) {
      setError(err instanceof Error ? err.message : "İşlem başarısız");
    } finally {
      setBulkBusy(false);
    }
  }

  const startImport = () =>
    runBulk(async () => {
      const path = await pickFile.importSource();
      if (!path) return;
      const preview = await api.productsImportPreview(path);
      setImportPath(path);
      setImportPreview(preview);
    });

  function closeImport() {
    setImportPath(null);
    setImportPreview(null);
  }

  const applyImport = () =>
    runBulk(async () => {
      if (!importPath) return;
      try {
        const r = await api.productsImportApply(importPath);
        const parts = [`${r.added} yeni`, `${r.updated} güncellendi`];
        if (r.reactivated > 0) parts.push(`${r.reactivated} geri getirildi`);
        if (r.skipped > 0) parts.push(`${r.skipped} satır atlandı`);
        setMessage(`İçe aktarıldı: ${parts.join(", ")}`);
        closeImport();
        await load();
      } catch (err) {
        closeImport();
        throw err;
      }
    });

  const exportProducts = () =>
    runBulk(async () => {
      const dest = await pickFile.exportTarget();
      if (!dest) return;
      const count = await api.productsExport(dest);
      setMessage(`${count} ürün dışa aktarıldı: ${dest}`);
    });

  const downloadTemplate = () =>
    runBulk(async () => {
      const dest = await pickFile.templateTarget();
      if (!dest) return;
      await api.productsExportTemplate(dest);
      setMessage(`Şablon kaydedildi: ${dest}`);
    });

  async function exitEditMode() {
    await logout();
    resetForm();
    setScanBarcode("");
    setMessage(null);
    setError(null);
  }

  const searching = search.trim().length > 0;

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

      {(error || message) && (
        <div className="products-flash">
          {error && <p className="flash error">{error}</p>}
          {message && <p className="flash ok">{message}</p>}
        </div>
      )}

      <div className="products-layout">
        {loggedIn && (
          <aside className="products-side">
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

            <section className="bulk-panel card-panel">
              <h2 className="panel-title">Toplu işlemler</h2>
              <p className="muted">
                Excel veya CSV dosyasından çok sayıda ürünü bir kerede ekleyin ya da
                fiyatları güncelleyin. Önce önizleme gösterilir.
              </p>
              <div className="bulk-actions">
                <button
                  type="button"
                  className="btn primary"
                  disabled={bulkBusy}
                  onClick={() => void startImport()}
                >
                  Excel/CSV içe aktar
                </button>
                <button
                  type="button"
                  className="btn"
                  disabled={bulkBusy}
                  onClick={() => void exportProducts()}
                >
                  Dışa aktar
                </button>
                <button
                  type="button"
                  className="btn ghost"
                  disabled={bulkBusy}
                  onClick={() => void downloadTemplate()}
                >
                  Şablon indir
                </button>
              </div>
            </section>
          </aside>
        )}

        <section className="products-list card-panel">
          <div className="search-bar">
            <input
              type="search"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setSearchMiss(false);
              }}
              onKeyDown={onSearchKeyDown}
              placeholder="Ürün adı veya barkod ara…"
              aria-label="Ürün ara"
              autoComplete="off"
            />
            <span className="search-count muted">
              {searching
                ? `${products.length} üründen ${filtered.length}'${possessiveSuffixTr(filtered.length)}`
                : `${products.length} ürün`}
            </span>
          </div>
          {searchMiss && <p className="flash error">Ürün bulunamadı</p>}

          <div className="table-wrap">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Barkod</th>
                  <th>Ad</th>
                  <th>Fiyat</th>
                  {loggedIn && <th></th>}
                </tr>
              </thead>
              <tbody ref={tableRef}>
                {products.length === 0 ? (
                  <tr>
                    <td colSpan={loggedIn ? 4 : 3} className="muted">
                      Henüz ürün yok
                      {loggedIn
                        ? ". Düzenleme modunda ilk ürünü ekleyin."
                        : ". Sahip düzenleme modunda ekleyebilir."}
                    </td>
                  </tr>
                ) : filtered.length === 0 ? (
                  <tr>
                    <td colSpan={loggedIn ? 4 : 3} className="muted">
                      Aramaya uyan ürün yok.
                    </td>
                  </tr>
                ) : (
                  filtered.map((p) => (
                    <tr
                      key={p.id}
                      data-product-id={p.id}
                      className={p.id === highlightId ? "row-highlight" : undefined}
                    >
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
        </section>
      </div>

      <ImportPreviewDialog
        preview={importPreview}
        applying={bulkBusy}
        onApply={() => void applyImport()}
        onClose={closeImport}
      />
    </div>
  );
}
