import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useApi } from "@/hooks/use-api";
import type { ProductivityData } from "@/lib/types";
import { formatDuration } from "@/lib/format";
import { getProductivityColor } from "@/lib/colors";

function getProductivityLabel(score: number): string {
  if (score >= 80) return "Excellent";
  if (score >= 65) return "Good";
  if (score >= 50) return "Average";
  if (score >= 35) return "Below average";
  return "Low";
}

export function ProductivityGauge() {
  const { data, loading } = useApi<ProductivityData>("/api/productivity");

  const score = data?.score ?? 0;
  const color = getProductivityColor(score);
  const label = getProductivityLabel(score);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Productivity Breakdown</CardTitle>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="h-[200px] bg-muted animate-pulse rounded" />
        ) : !data ? (
          <div className="py-8 text-center text-muted-foreground">No data</div>
        ) : (
          <div className="space-y-4">
            {/* Score display */}
            <div className="flex items-center justify-center">
              <div className="relative w-32 h-32">
                <svg viewBox="0 0 100 100" className="transform -rotate-90">
                  <circle
                    cx="50"
                    cy="50"
                    r="40"
                    fill="none"
                    stroke="currentColor"
                    className="text-muted"
                    strokeWidth="8"
                  />
                  <circle
                    cx="50"
                    cy="50"
                    r="40"
                    fill="none"
                    stroke={color}
                    strokeWidth="8"
                    strokeLinecap="round"
                    strokeDasharray={`${score * 2.51} 251`}
                  />
                </svg>
                <div className="absolute inset-0 flex flex-col items-center justify-center">
                  <span className="text-2xl font-bold" style={{ color }}>
                    {score}
                  </span>
                  <span className="text-[10px] text-muted-foreground">{label}</span>
                </div>
              </div>
            </div>

            {/* Breakdown bars */}
            <div className="space-y-2">
              <BreakdownBar
                label="Productive"
                seconds={data.productive}
                total={data.total}
                color="#10b981"
              />
              <BreakdownBar
                label="Neutral"
                seconds={data.neutral}
                total={data.total}
                color="#f59e0b"
              />
              <BreakdownBar
                label="Distracting"
                seconds={data.distracting}
                total={data.total}
                color="#ef4444"
              />
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function BreakdownBar({
  label,
  seconds,
  total,
  color,
}: {
  label: string;
  seconds: number;
  total: number;
  color: string;
}) {
  const pct = total > 0 ? (seconds / total) * 100 : 0;

  return (
    <div className="space-y-1">
      <div className="flex justify-between text-sm">
        <span className="text-muted-foreground">{label}</span>
        <span className="font-medium">{formatDuration(seconds)}</span>
      </div>
      <div className="h-2 bg-muted rounded-full overflow-hidden">
        <div
          className="h-full rounded-full transition-all"
          style={{ width: `${pct}%`, backgroundColor: color }}
        />
      </div>
    </div>
  );
}
