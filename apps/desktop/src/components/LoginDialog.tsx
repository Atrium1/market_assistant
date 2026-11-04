import { FormEvent, useEffect, useRef, useState } from "react";

type Props = {
  open: boolean;
  title?: string;
  onClose: () => void;
  onSubmit: (username: string, password: string) => Promise<void>;
};

export function LoginDialog({
  open,
  title = "Sahip girişi",
  onClose,
  onSubmit,
}: Props) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const userRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (open) {
      setUsername("");
      setPassword("");
      setError(null);
      requestAnimationFrame(() => userRef.current?.focus());
    }
  }, [open]);

  if (!open) return null;

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      await onSubmit(username.trim(), password);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Giriş başarısız");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="modal-backdrop" role="presentation" onClick={onClose}>
      <div
        className="modal-card"
        role="dialog"
        aria-modal="true"
        aria-labelledby="login-title"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="login-title">{title}</h2>
        <p className="muted">
          Ürün düzenleme ve ayarlar için kullanıcı adı ve şifre girin.
        </p>
        <form onSubmit={(e) => void handleSubmit(e)}>
          <div className="form-row">
            <label htmlFor="login-username">Kullanıcı adı</label>
            <input
              id="login-username"
              ref={userRef}
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              autoComplete="username"
            />
          </div>
          <div className="form-row">
            <label htmlFor="login-password">Şifre</label>
            <input
              id="login-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="current-password"
            />
          </div>
          {error && <p className="flash error">{error}</p>}
          <div className="form-actions">
            <button type="button" className="btn ghost" onClick={onClose}>
              Vazgeç
            </button>
            <button type="submit" className="btn primary" disabled={loading}>
              Giriş yap
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
