import { useContext } from "react";

import {
  DatabaseContext,
  type DatabaseContextValue,
} from "@/contexts/databaseContext";

export function useDatabase(): DatabaseContextValue {
  const context = useContext(DatabaseContext);
  if (!context) {
    throw new Error("useDatabase must be used within DatabaseProvider");
  }
  return context;
}
