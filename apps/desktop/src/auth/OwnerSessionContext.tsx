import {
  ReactNode,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { api, isAuthError } from "../lib/api";
import type { SessionStatus } from "../types";
import { LoginDialog } from "../components/LoginDialog";

type OwnerSessionContextValue = {
  loggedIn: boolean;
  viaFactory: boolean;
  usernameDisplay: string | null;
  refresh: () => Promise<void>;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  openLogin: (onSuccess?: () => void) => void;
  /** Run an owner-only call; if the session is missing or expired, open login and retry after it. */
  requireOwner: (
    action: () => Promise<void>,
    onError?: (err: unknown) => void,
  ) => Promise<void>;
};

const OwnerSessionContext = createContext<OwnerSessionContextValue | null>(
  null,
);

export function OwnerSessionProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<SessionStatus>({
    logged_in: false,
    via_factory: false,
    username_display: null,
  });
  const [loginOpen, setLoginOpen] = useState(false);
  const [pendingSuccess, setPendingSuccess] = useState<(() => void) | null>(
    null,
  );

  const refresh = useCallback(async () => {
    try {
      const next = await api.authSessionStatus();
      setStatus(next);
    } catch {
      setStatus({
        logged_in: false,
        via_factory: false,
        username_display: null,
      });
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), 60_000);
    return () => window.clearInterval(id);
  }, [refresh]);

  const login = useCallback(async (username: string, password: string) => {
    const next = await api.authLogin(username, password);
    setStatus(next);
  }, []);

  const logout = useCallback(async () => {
    await api.authLogout();
    setStatus({
      logged_in: false,
      via_factory: false,
      username_display: null,
    });
  }, []);

  const openLogin = useCallback((onSuccess?: () => void) => {
    setPendingSuccess(() => onSuccess ?? null);
    setLoginOpen(true);
  }, []);

  const requireOwner = useCallback(
    async (action: () => Promise<void>, onError?: (err: unknown) => void) => {
      const report = (err: unknown) => {
        if (onError) onError(err);
        else throw err;
      };
      try {
        await action();
      } catch (err) {
        if (!isAuthError(err)) return report(err);
        await refresh();
        openLogin(() => {
          action().catch((retryErr) => {
            if (!isAuthError(retryErr)) onError?.(retryErr);
          });
        });
      }
    },
    [refresh, openLogin],
  );

  const value = useMemo(
    () => ({
      loggedIn: status.logged_in,
      viaFactory: status.via_factory,
      usernameDisplay: status.username_display,
      refresh,
      login,
      logout,
      openLogin,
      requireOwner,
    }),
    [status, refresh, login, logout, openLogin, requireOwner],
  );

  return (
    <OwnerSessionContext.Provider value={value}>
      {children}
      <LoginDialog
        open={loginOpen}
        onClose={() => {
          setLoginOpen(false);
          setPendingSuccess(null);
        }}
        onSubmit={async (u, p) => {
          await login(u, p);
          pendingSuccess?.();
          setPendingSuccess(null);
        }}
      />
    </OwnerSessionContext.Provider>
  );
}

export function useOwnerSession() {
  const ctx = useContext(OwnerSessionContext);
  if (!ctx) {
    throw new Error("useOwnerSession must be used within OwnerSessionProvider");
  }
  return ctx;
}
