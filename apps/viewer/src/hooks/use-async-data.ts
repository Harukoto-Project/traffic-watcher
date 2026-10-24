import { useEffect, useRef, useState } from "react";

import { toAppError, type AppError } from "@/services/AppError";

export interface AsyncState<T> {
  data: T | null;
  loading: boolean;
  error: AppError | null;
  reload: () => void;
}

interface Settled<T> {
  key: string;
  data: T | null;
  error: AppError | null;
}

export function useAsyncData<T>(
  loader: () => Promise<T>,
  deps: readonly unknown[],
): AsyncState<T> {
  const [nonce, setNonce] = useState(0);
  const [settled, setSettled] = useState<Settled<T>>({
    key: "",
    data: null,
    error: null,
  });
  const loaderRef = useRef(loader);
  const key = JSON.stringify([...deps, nonce]);

  useEffect(() => {
    loaderRef.current = loader;
  });

  useEffect(() => {
    let cancelled = false;
    loaderRef
      .current()
      .then((result) => {
        if (!cancelled) {
          setSettled({ key, data: result, error: null });
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setSettled((prev) => ({
            key,
            data: prev.data,
            error: toAppError(err),
          }));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [key]);

  return {
    data: settled.data,
    loading: settled.key !== key,
    error: settled.error,
    reload: () => setNonce((value) => value + 1),
  };
}
