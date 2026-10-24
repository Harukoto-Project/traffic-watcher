import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import { readStored, writeStored } from "@/lib/browserStorage";
import { toAppError, type AppError } from "@/services/AppError";
import {
  findDefaultDatabase,
  loadDatabaseInfo,
  openDatabase,
  pickDatabaseFile,
} from "@/services/databaseService";
import type { DatabaseInfo } from "@/types/database";

import { DatabaseContext, type DatabaseStatus } from "./databaseContext";

const LAST_PATH_KEY = "viewer-last-database";

async function collectStartupCandidates(): Promise<string[]> {
  const candidates: string[] = [];
  const stored = readStored(LAST_PATH_KEY);
  if (stored) {
    candidates.push(stored);
  }
  try {
    const found = await findDefaultDatabase();
    if (found && found !== stored) {
      candidates.push(found);
    }
  } catch {
    return candidates;
  }
  return candidates;
}

export function DatabaseProvider({ children }: { children: ReactNode }) {
  const [info, setInfo] = useState<DatabaseInfo | null>(null);
  const [status, setStatus] = useState<DatabaseStatus>("loading");
  const [error, setError] = useState<AppError | null>(null);
  const [revision, setRevision] = useState(0);

  const accept = useCallback((next: DatabaseInfo) => {
    setInfo(next);
    setStatus("ready");
    setError(null);
    setRevision((value) => value + 1);
  }, []);

  const openPath = useCallback(
    async (path: string) => {
      try {
        const next = await openDatabase(path);
        writeStored(LAST_PATH_KEY, path);
        accept(next);
      } catch (err) {
        setError(toAppError(err));
        setStatus((prev) => (prev === "loading" ? "empty" : prev));
      }
    },
    [accept],
  );

  const openWithDialog = useCallback(async () => {
    try {
      const path = await pickDatabaseFile();
      if (path) {
        await openPath(path);
      }
    } catch (err) {
      setError(toAppError(err));
    }
  }, [openPath]);

  const reload = useCallback(async () => {
    try {
      const next = await loadDatabaseInfo();
      if (next) {
        accept(next);
      }
    } catch (err) {
      setError(toAppError(err));
    }
  }, [accept]);

  const dismissError = useCallback(() => setError(null), []);

  useEffect(() => {
    let cancelled = false;
    async function startup() {
      const candidates = await collectStartupCandidates();
      for (const path of candidates) {
        try {
          const next = await openDatabase(path);
          if (!cancelled) {
            accept(next);
          }
          return;
        } catch {
          continue;
        }
      }
      if (!cancelled) {
        setStatus("empty");
      }
    }
    void startup();
    return () => {
      cancelled = true;
    };
  }, [accept]);

  const value = useMemo(
    () => ({
      status,
      info,
      error,
      revision,
      openPath,
      openWithDialog,
      reload,
      dismissError,
    }),
    [
      status,
      info,
      error,
      revision,
      openPath,
      openWithDialog,
      reload,
      dismissError,
    ],
  );

  return (
    <DatabaseContext.Provider value={value}>
      {children}
    </DatabaseContext.Provider>
  );
}
