import { useCallback, useEffect, useRef } from "react";

/** Keep barcode input focused for USB keyboard-wedge scanners. */
export function useBarcodeFocus(enabled = true) {
  const ref = useRef<HTMLInputElement | null>(null);

  const focusBarcode = useCallback(() => {
    if (!enabled) return;
    requestAnimationFrame(() => {
      ref.current?.focus();
      ref.current?.select();
    });
  }, [enabled]);

  useEffect(() => {
    focusBarcode();
  }, [focusBarcode]);

  useEffect(() => {
    if (!enabled) return;
    const onWindowFocus = () => focusBarcode();
    window.addEventListener("focus", onWindowFocus);
    return () => window.removeEventListener("focus", onWindowFocus);
  }, [enabled, focusBarcode]);

  return { ref, focusBarcode };
}
