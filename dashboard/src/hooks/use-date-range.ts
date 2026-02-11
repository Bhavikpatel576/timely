import { createContext, useContext } from "react";
import type { DateRange } from "@/lib/types";

export interface DateRangeContextType {
  range: DateRange;
  setRange: (range: DateRange) => void;
  refreshKey: number;
  triggerRefresh: () => void;
}

export const DateRangeContext = createContext<DateRangeContextType | null>(null);

export function useDateRange(): DateRangeContextType {
  const ctx = useContext(DateRangeContext);
  if (!ctx) throw new Error("useDateRange must be used within DateRangeProvider");
  return ctx;
}
