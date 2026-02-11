import { useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from "recharts";
import { useApi } from "@/hooks/use-api";
import { useDateRange } from "@/hooks/use-date-range";
import type { TrendBucket } from "@/lib/types";
import { getCategoryColor } from "@/lib/colors";
import { formatDuration } from "@/lib/format";

export function HourlyActivity() {
  const { range } = useDateRange();
  const { data, loading } = useApi<TrendBucket[]>("/api/trends", { interval: "hour" });
  const isMultiDay = range.from !== range.to;

  const chartData = useMemo(() => {
    if (!data || data.length === 0) return [];

    // Build a 0-23 hour array
    const hours: { hour: number; label: string; seconds: number; dominantCategory: string }[] = [];
    for (let h = 0; h < 24; h++) {
      hours.push({ hour: h, label: `${h.toString().padStart(2, "0")}`, seconds: 0, dominantCategory: "uncategorized" });
    }

    for (const bucket of data) {
      // bucket.bucket is like "2025-01-15T14:00"
      const match = bucket.bucket.match(/T(\d{2}):/);
      if (!match) continue;
      const hour = parseInt(match[1], 10);
      if (hour >= 0 && hour < 24) {
        hours[hour].seconds += bucket.total_seconds;
        // Find dominant category
        let maxSecs = 0;
        for (const [cat, secs] of Object.entries(bucket.categories)) {
          if (secs > maxSecs) {
            maxSecs = secs;
            hours[hour].dominantCategory = cat;
          }
        }
      }
    }

    return hours;
  }, [data]);

  return (
    <Card>
      <CardHeader>
        <div>
          <CardTitle>Activity by Hour</CardTitle>
          {isMultiDay && (
            <p className="text-xs text-muted-foreground mt-1">Aggregated across selected days</p>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="h-[300px] bg-muted animate-pulse rounded" />
        ) : chartData.length === 0 ? (
          <div className="h-[300px] flex items-center justify-center text-muted-foreground">
            No data for this period
          </div>
        ) : (
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={chartData}>
              <XAxis
                dataKey="label"
                tick={{ fontSize: 11 }}
                interval={2}
              />
              <YAxis
                tick={{ fontSize: 11 }}
                tickFormatter={(v) => `${Math.round(v / 60)}m`}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'var(--color-card)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '0.5rem',
                  color: 'var(--color-foreground)',
                }}
                labelStyle={{ color: 'var(--color-foreground)' }}
                formatter={(value: unknown) => [formatDuration(Number(value)), "Active"]}
                labelFormatter={(label) => `${label}:00`}
              />
              <Bar dataKey="seconds" radius={[2, 2, 0, 0]}>
                {chartData.map((entry, i) => (
                  <Cell key={i} fill={getCategoryColor(entry.dominantCategory)} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
