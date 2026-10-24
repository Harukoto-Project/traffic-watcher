import { createContext } from "react";

import type { AppError } from "@/services/AppError";
import type { DatabaseInfo } from "@/types/database";

export type DatabaseStatus = "loading" | "ready" | "empty";

export interface DatabaseContextValue {
  status: DatabaseStatus;
  info: DatabaseInfo | null;
  error: AppError | null;
  revision: number;
  openPath: (path: string) => Promise<void>;
  openWithDialog: () => Promise<void>;
  reload: () => Promise<void>;
  dismissError: () => void;
}

export const DatabaseContext = createContext<DatabaseContextValue | null>(null);
