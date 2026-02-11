import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";
import { useApi } from "@/hooks/use-api";
import type { SummaryResponse } from "@/lib/types";
import { getCategoryColor } from "@/lib/colors";
import { formatDuration } from "@/lib/format";
import { isInappropriate } from "@/lib/inappropriate";

export function CategoryPieChart() {
  const { data, loading } = useApi<SummaryResponse>("/api/summary", { groupBy: "category" });

  const chartData = (data?.groups || []).slice(0, 8).map((g) => {
    const shortName = g.name.split("/").pop() || g.name;
    return {
      name: isInappropriate(g.name) ? "Flagged" : shortName,
      fullName: g.name,
      value: g.seconds,
      pct: g.pct,
    };
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>Category Distribution</CardTitle>
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
            <PieChart>
              <Pie
                data={chartData}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={100}
                paddingAngle={2}
                dataKey="value"
              >
                {chartData.map((entry) => (
                  <Cell key={entry.fullName} fill={getCategoryColor(entry.fullName)} />
                ))}
              </Pie>
              <Tooltip
                formatter={(value: unknown) => [formatDuration(Number(value)), "Time"]}
              />
              <Legend
                formatter={(value: string) => (
                  <span className="text-xs">{value}</span>
                )}
              />
            </PieChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}
