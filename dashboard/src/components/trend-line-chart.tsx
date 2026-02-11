import { useState, useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  ComposedChart,
  Area,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { useApi } from "@/hooks/use-api";
import { toDateString } from "@/lib/format";
import type { TrendBucket } from "@/lib/types";
import { getCategoryColor } from "@/lib/colors";
import { isInappropriate } from "@/lib/inappropriate";

type TrendRange = "7d" | "30d" | "custom";

export function TrendLineChart() {
  const [trendRange, setTrendRange] = useState<TrendRange>("7d");

  const overrideDateRange = useMemo(() => {
    if (trendRange === "custom") return undefined;
    const days = trendRange === "7d" ? 7 : 30;
    const to = new Date();
    const from = new Date(to.getTime() - days * 86400000);
    return { from: toDateString(from), to: toDateString(to) };
  }, [trendRange]);

  const { data, loading } = useApi<TrendBucket[]>(
    "/api/trends",
    { interval: "day" },
    { overrideDateRange }
  );

  const { chartData, categoryKeys } = useMemo(() => {
    if (!data || data.length === 0) return { chartData: [], categoryKeys: [] as string[] };

    // Collect all unique categories
    const catSet = new Set<string>();
    for (const bucket of data) {
      for (const cat of Object.keys(bucket.categories)) {
        catSet.add(cat);
      }
    }
    const categoryKeys = Array.from(catSet);

    const chartData = data.map((bucket) => {
      const entry: Record<string, number | string> = {
        bucket: bucket.bucket,
        productivity: bucket.productivity,
      };
      for (const cat of categoryKeys) {
        // Convert to hours
        entry[cat] = Math.round(((bucket.categories[cat] || 0) / 3600) * 10) / 10;
      }
      return entry;
    });

    return { chartData, categoryKeys };
  }, [data]);

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Activity Trends</CardTitle>
          <div className="flex gap-1">
            {(["7d", "30d", "custom"] as const).map((r) => (
              <Button
                key={r}
                variant={trendRange === r ? "default" : "ghost"}
                size="sm"
                className="h-7 text-xs px-2"
                onClick={() => setTrendRange(r)}
              >
                {r === "7d" ? "7 Days" : r === "30d" ? "30 Days" : "Date Range"}
              </Button>
            ))}
          </div>
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
            <ComposedChart data={chartData}>
              <XAxis
                dataKey="bucket"
                tick={{ fontSize: 11 }}
                interval="preserveStartEnd"
              />
              <YAxis
                yAxisId="hours"
                tick={{ fontSize: 11 }}
                label={{ value: "Hours", angle: -90, position: "insideLeft", fontSize: 11 }}
              />
              <YAxis
                yAxisId="score"
                orientation="right"
                domain={[0, 100]}
                tick={{ fontSize: 11 }}
                label={{ value: "Score", angle: 90, position: "insideRight", fontSize: 11 }}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'var(--color-card)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '0.5rem',
                  color: 'var(--color-foreground)',
                }}
                labelStyle={{ color: 'var(--color-foreground)' }}
              />
              <Legend />
              {categoryKeys.map((cat) => (
                <Area
                  key={cat}
                  yAxisId="hours"
                  type="monotone"
                  dataKey={cat}
                  stackId="1"
                  fill={getCategoryColor(cat)}
                  stroke={getCategoryColor(cat)}
                  fillOpacity={0.6}
                  name={isInappropriate(cat) ? "Flagged" : (cat.split("/").pop() || cat)}
                />
              ))}
              <Line
                yAxisId="score"
                type="monotone"
                dataKey="productivity"
                stroke="#8b5cf6"
                strokeWidth={2}
                dot={false}
                name="Productivity"
              />
            </ComposedChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
