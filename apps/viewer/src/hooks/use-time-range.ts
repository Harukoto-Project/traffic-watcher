import { useContext } from "react";

import {
  TimeRangeContext,
  type TimeRangeContextValue,
} from "@/contexts/timeRangeContext";

export function useTimeRange(): TimeRangeContextValue {
  const context = useContext(TimeRangeContext);
  if (!context) {
    throw new Error("useTimeRange must be used within TimeRangeProvider");
  }
  return context;
}
