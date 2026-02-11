import { useState, useEffect, useCallback, useMemo } from "react";
import { fetchApi } from "@/lib/api";
import { useDateRange } from "./use-date-range";

export function useApi<T>(
  path: string,
  extraParams?: Record<string, string>,
  options?: {
    refreshInterval?: number;
    includeDateRange?: boolean;
    overrideDateRange?: { from: string; to: string };
  }
) {
  const { range, refreshKey } = useDateRange();
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  const includeDateRange = options?.includeDateRange !== false;
  const overrideFrom = options?.overrideDateRange?.from;
  const overrideTo = options?.overrideDateRange?.to;

  const paramsKey = useMemo(() => JSON.stringify(extraParams ?? {}), [extraParams]);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const dateFrom = overrideFrom ?? range.from;
      const dateTo = overrideTo ?? range.to;
      const params: Record<string, string> = {
        ...(includeDateRange ? { from: dateFrom, to: dateTo } : {}),
        ...extraParams,
      };
      const result = await fetchApi<T>(path, params);
      setData(result);
      setLastUpdated(new Date());
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  }, [path, range.from, range.to, includeDateRange, overrideFrom, overrideTo, refreshKey, paramsKey]);

  useEffect(() => {
    load();

    if (options?.refreshInterval) {
      const interval = setInterval(load, options.refreshInterval);
      return () => clearInterval(interval);
    }
  }, [load, options?.refreshInterval]);

  return { data, loading, error, lastUpdated, refetch: load };
}
