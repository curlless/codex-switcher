import { useEffect, useRef, useState } from "react";
import type { Toast } from "../components/ToastContainer";

const TOAST_TTL_MS = 5000;

export function useToasts() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const toastCounterRef = useRef(0);
  const timeoutIdsRef = useRef<number[]>([]);

  useEffect(() => {
    return () => {
      timeoutIdsRef.current.forEach((id) => window.clearTimeout(id));
      timeoutIdsRef.current = [];
    };
  }, []);

  function addToast(
    status: Toast["status"],
    title: string,
    detail: string,
    hints: string[] = [],
  ) {
    toastCounterRef.current += 1;
    const id = toastCounterRef.current;

    setToasts((prev) => [...prev, { id, status, title, detail, hints }]);

    const timeoutId = window.setTimeout(() => {
      setToasts((prev) => prev.filter((toast) => toast.id !== id));
      timeoutIdsRef.current = timeoutIdsRef.current.filter(
        (savedId) => savedId !== timeoutId,
      );
    }, TOAST_TTL_MS);

    timeoutIdsRef.current.push(timeoutId);
  }

  function removeToast(id: number) {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }

  function clearToasts() {
    setToasts([]);
  }

  return {
    toasts,
    addToast,
    removeToast,
    clearToasts,
  };
}
