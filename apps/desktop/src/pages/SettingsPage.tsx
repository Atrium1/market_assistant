import { FormEvent, useCallback, useEffect, useState } from "react";
import { useOwnerSession } from "../auth/OwnerSessionContext";
import { api } from "../lib/api";

export function SettingsPage() {
  const { loggedIn, viaFactory, usernameDisplay, openLogin, logout } =
    useOwnerSession();
  const [displayName, setDisplayName] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [passwordConfirm, setPasswordConfirm] = useState("");
  const [credsSet, setCredsSet] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    if (!loggedIn) return;
    setError(null);
    try {
      const s = await api.settingsGet();
      setDisplayName(s.store_display_name ?? "");
      setUsername(s.shop_username ?? "");
      setCredsSet(s.shop_credentials_set);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Ayarlar yüklenemedi");
    }
  }, [loggedIn]);

  useEffect(() => {
    void load();
  }, [load]);

  async function saveStoreName(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setMessage(null);
    setLoading(true);
    try {
      await api.settingsSetStoreName(displayName);
      setMessage("Mağaza adı kaydedildi");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Kayıt başarısız");
    } finally {
      setLoading(false);
    }
  }

  async function saveCredentials(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setMessage(null);
    if (password !== passwordConfirm) {
      setError("Şifreler eşleşmiyor");
      return;
    }
    setLoading(true);
    try {
      await api.settingsSetOwnerCredentials({
        username: username.trim(),
        password,
        password_confirm: passwordConfirm,
      });
      setPassword("");
      setPasswordConfirm("");
      setCredsSet(true);
      setMessage("Dükkan kullanıcı adı ve şifresi kaydedildi");
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Kayıt başarısız");
    } finally {
      setLoading(false);
    }
  }

  if (!loggedIn) {
    return (
      <div className="page settings-page">
        <header className="page-header">
          <h1>Ayarlar</h1>
          <p>Mağaza adı ve dükkan hesabı için sahip girişi gerekir.</p>
        </header>
        <div className="card-panel settings-locked">
          <p>Ayarları görmek için giriş yapın.</p>
          <button
            type="button"
            className="btn primary"
            onClick={() => openLogin()}
          >
            Giriş yap
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="page settings-page">
      <header className="page-header products-header">
        <div>
          <h1>Ayarlar</h1>
          <p>
            Oturum: <strong>{usernameDisplay}</strong>
            {viaFactory ? " (fabrika kurtarma girişi)" : ""}
          </p>
        </div>
        <button type="button" className="btn" onClick={() => void logout()}>
          Çıkış
        </button>
      </header>

      <form className="card-panel settings-form" onSubmit={(e) => void saveStoreName(e)}>
        <h2 className="panel-title">Mağaza adı</h2>
        <div className="form-row">
          <label htmlFor="store-name">Görünen ad</label>
          <input
            id="store-name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            autoComplete="off"
          />
        </div>
        <div className="form-actions">
          <button type="submit" className="btn primary" disabled={loading}>
            Adı kaydet
          </button>
        </div>
      </form>

      <form
        className="card-panel settings-form"
        onSubmit={(e) => void saveCredentials(e)}
      >
        <h2 className="panel-title">Dükkan kullanıcı adı ve şifre</h2>
        <p className="muted">
          {credsSet
            ? "Mevcut dükkan hesabını güncellersiniz. Unutursanız fabrika kurtarma hesabı ile giriş yapıp buradan yenileyebilirsiniz."
            : "Henüz dükkan hesabı yok. İlk hesabınızı burada oluşturun."}
        </p>
        <p className="muted note-factory">
          Fabrika (kurtarma) kullanıcı adı/şifresi bu ekranda gösterilmez ve
          değiştirilemez.
        </p>
        <div className="form-row">
          <label htmlFor="shop-username">Kullanıcı adı</label>
          <input
            id="shop-username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            autoComplete="username"
          />
        </div>
        <div className="form-row">
          <label htmlFor="shop-password">Yeni şifre</label>
          <input
            id="shop-password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="new-password"
          />
        </div>
        <div className="form-row">
          <label htmlFor="shop-password-confirm">Şifre tekrar</label>
          <input
            id="shop-password-confirm"
            type="password"
            value={passwordConfirm}
            onChange={(e) => setPasswordConfirm(e.target.value)}
            autoComplete="new-password"
          />
        </div>
        <div className="form-actions">
          <button type="submit" className="btn primary" disabled={loading}>
            Hesabı kaydet
          </button>
        </div>
      </form>

      {error && <p className="flash error">{error}</p>}
      {message && <p className="flash ok">{message}</p>}
    </div>
  );
}
