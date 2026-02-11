import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from "recharts";
import { useApi } from "@/hooks/use-api";
import type { SummaryResponse } from "@/lib/types";
import { getCategoryColor } from "@/lib/colors";
import { formatDuration } from "@/lib/format";
import { isInappropriate } from "@/lib/inappropriate";

export function CategoryBarChart() {
  const { data, loading } = useApi<SummaryResponse>("/api/summary", { groupBy: "category" });

  const chartData = (data?.groups || []).slice(0, 10).map((g) => {
    const shortName = g.name.split("/").pop() || g.name;
    return {
      name: isInappropriate(g.name) ? "Flagged" : shortName,
      fullName: g.name,
      seconds: g.seconds,
      time: g.time,
      pct: g.pct,
    };
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>Time by Category</CardTitle>
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
            <BarChart data={chartData} layout="vertical" margin={{ left: 20, right: 20 }}>
              <XAxis type="number" hide />
              <YAxis type="category" dataKey="name" width={120} tick={{ fontSize: 12 }} />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'var(--color-card)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '0.5rem',
                  color: 'var(--color-foreground)',
                }}
                labelStyle={{ color: 'var(--color-foreground)' }}
                formatter={(value: unknown) => [formatDuration(Number(value)), "Time"]}
                labelFormatter={(label: unknown) => {
                  const s = String(label);
                  const item = chartData.find((d) => d.name === s);
                  return item ? item.fullName : s;
                }}
              />
              <Bar dataKey="seconds" radius={[0, 4, 4, 0]}>
                {chartData.map((entry) => (
                  <Cell key={entry.fullName} fill={getCategoryColor(entry.fullName)} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
