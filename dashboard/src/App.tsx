import { useState, useEffect, useCallback, useRef } from "react";
import { Clock, TrendingUp, LayoutGrid, Activity, Moon, Sun, Settings, Info } from "lucide-react";
import { Button } from "@/components/ui/button";
import { DateRangeContext } from "@/hooks/use-date-range";
import { useApi } from "@/hooks/use-api";
import { useDateRange } from "@/hooks/use-date-range";
import { DateRangePicker } from "@/components/date-range-picker";
import { StatCard } from "@/components/stat-card";
import { CategoryBarChart } from "@/components/category-bar-chart";
import { HourlyActivity } from "@/components/hourly-activity";
import { AppBreakdownTable } from "@/components/app-breakdown-table";
import { TimelineBlocks } from "@/components/timeline-blocks";
import { ProductivityGauge } from "@/components/productivity-gauge";
import { TrendLineChart } from "@/components/trend-line-chart";
import { RulesPanel } from "@/components/rules-panel";
import { FlaggedContentBanner } from "@/components/flagged-content-banner";
import { FlaggedActivitySection } from "@/components/flagged-activity-section";
import { todayString, toDateString, formatDuration, formatRelativeTime } from "@/lib/format";
import { isInappropriate } from "@/lib/inappropriate";
import type { DateRange, SummaryResponse, ProductivityData, CurrentActivity } from "@/lib/types";

function getProductivityLabel(score: number): string {
  if (score >= 80) return "Excellent";
  if (score >= 65) return "Good";
  if (score >= 50) return "Average";
  if (score >= 35) return "Below average";
  return "Low";
}

function DashboardContent({
  bannerDismissed,
  onDismissBanner,
  onOpenRules,
}: {
  bannerDismissed: boolean;
  onDismissBanner: () => void;
  onOpenRules: () => void;
}) {
  const { range } = useDateRange();
  const isToday = range.label === "Today" || (range.from === todayString() && range.to === todayString());

  const { data: summary, loading: summaryLoading, lastUpdated } = useApi<SummaryResponse>("/api/summary", {
    groupBy: "category",
  });
  const { data: productivity, loading: productivityLoading } =
    useApi<ProductivityData>("/api/productivity");
  const { data: current, loading: currentLoading } = useApi<CurrentActivity | null>(
    "/api/current",
    undefined,
    { refreshInterval: 60000, includeDateRange: false }
  );

  // Fetch yesterday's data for comparison (only when viewing "Today")
  const yesterday = toDateString(new Date(Date.now() - 86400000));
  const { data: yesterdaySummary } = useApi<SummaryResponse>(
    "/api/summary",
    { groupBy: "category" },
    { overrideDateRange: isToday ? { from: yesterday, to: yesterday } : undefined }
  );
  const { data: yesterdayProductivity } = useApi<ProductivityData>(
    "/api/productivity",
    undefined,
    { overrideDateRange: isToday ? { from: yesterday, to: yesterday } : undefined }
  );

  // Ticking "Updated X ago" display
  const [updatedText, setUpdatedText] = useState<string>("");
  useEffect(() => {
    if (!lastUpdated) return;
    setUpdatedText(formatRelativeTime(lastUpdated));
    const interval = setInterval(() => {
      setUpdatedText(formatRelativeTime(lastUpdated));
    }, 5000);
    return () => clearInterval(interval);
  }, [lastUpdated]);

  const topCategory = summary?.groups?.[0];
  const hasFlagged = summary?.groups?.some((g) => isInappropriate(g.name)) ?? false;
  const topCategoryLabel =
    topCategory && isInappropriate(topCategory.name) ? "Flagged" : topCategory?.name;
  const currentCategoryLabel =
    current && isInappropriate(current.category) ? "Flagged" : current?.category;

  // Compute uncategorized percentage
  const uncategorizedGroup = summary?.groups?.find((g) => g.name === "uncategorized");
  const uncategorizedPct = summary?.total_active_seconds && uncategorizedGroup
    ? Math.round((uncategorizedGroup.seconds / summary.total_active_seconds) * 100)
    : 0;

  // Compute vs-yesterday trends (only when viewing "Today")
  const timeTrend = (() => {
    if (!isToday || !summary || !yesterdaySummary) return null;
    const diff = summary.total_active_seconds - yesterdaySummary.total_active_seconds;
    if (diff === 0) return null;
    return { delta: `${diff > 0 ? "+" : ""}${formatDuration(Math.abs(diff))}`, positive: diff > 0 };
  })();
  const prodTrend = (() => {
    if (!isToday || !productivity || !yesterdayProductivity) return null;
    const diff = productivity.score - yesterdayProductivity.score;
    if (diff === 0) return null;
    return { delta: `${diff > 0 ? "+" : ""}${diff}pts`, positive: diff > 0 };
  })();

  const appTableRef = useRef<HTMLDivElement>(null);

  return (
    <div className="space-y-6">
      {/* Last updated indicator */}
      {updatedText && (
        <div className="text-xs text-muted-foreground text-right -mt-4">
          Updated {updatedText}
        </div>
      )}

      {/* Flagged content banner */}
      {hasFlagged && !bannerDismissed && (
        <FlaggedContentBanner
          onDismiss={onDismissBanner}
          onManageRules={onOpenRules}
        />
      )}

      {/* Flagged activity detail section */}
      <FlaggedActivitySection />

      {/* Uncategorized % banner */}
      {uncategorizedPct > 30 && (
        <div className="flex items-center gap-3 rounded-lg border border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-950/30 px-4 py-3">
          <Info className="h-4 w-4 text-blue-600 dark:text-blue-400 shrink-0" />
          <p className="text-sm text-blue-800 dark:text-blue-200 flex-1">
            {uncategorizedPct}% of your activity is uncategorized. Categorize your apps to get better insights.
          </p>
          <Button
            variant="outline"
            size="sm"
            className="border-blue-300 text-blue-700 hover:bg-blue-100 dark:border-blue-700 dark:text-blue-300 dark:hover:bg-blue-900"
            onClick={() => appTableRef.current?.scrollIntoView({ behavior: "smooth" })}
          >
            Review &amp; categorize
          </Button>
        </div>
      )}

      {/* Stat cards row */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="Total Active Time"
          value={summary?.total_active || "0m"}
          subtitle={`${summary?.groups?.length || 0} ${(summary?.groups?.length || 0) === 1 ? "category" : "categories"}`}
          icon={<Clock className="h-4 w-4" />}
          loading={summaryLoading}
          trend={timeTrend}
        />
        <StatCard
          title="Productivity Score"
          value={productivity ? `${productivity.score}/100` : "—"}
          subtitle={
            productivity
              ? `${getProductivityLabel(productivity.score)} · ${formatDuration(productivity.productive)} productive`
              : undefined
          }
          icon={<TrendingUp className="h-4 w-4" />}
          loading={productivityLoading}
          trend={prodTrend}
        />
        <StatCard
          title="Top Category"
          value={topCategoryLabel || "—"}
          subtitle={topCategory ? `${topCategory.time} (${topCategory.pct}%)` : undefined}
          icon={<LayoutGrid className="h-4 w-4" />}
          loading={summaryLoading}
        />
        <StatCard
          title="Current Activity"
          value={current?.app || "Idle"}
          subtitle={current ? `${currentCategoryLabel} · ${formatDuration(current.duration_seconds)}` : "No recent activity"}
          icon={<Activity className="h-4 w-4" />}
          loading={currentLoading}
        />
      </div>

      {/* Charts row: bar chart + hourly activity */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <CategoryBarChart />
        <HourlyActivity />
      </div>

      {/* App breakdown */}
      <div ref={appTableRef}>
        <AppBreakdownTable />
      </div>

      {/* Timeline */}
      <TimelineBlocks />

      {/* Productivity + Trends in one row */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <ProductivityGauge />
        <div className="lg:col-span-2">
          <TrendLineChart />
        </div>
      </div>
    </div>
  );
}

export default function App() {
  const [range, setRange] = useState<DateRange>({
    from: todayString(),
    to: todayString(),
    label: "Today",
  });

  const [refreshKey, setRefreshKey] = useState(0);
  const triggerRefresh = useCallback(() => setRefreshKey((k) => k + 1), []);

  // Auto-refresh all dashboard data every 30 seconds
  useEffect(() => {
    const interval = setInterval(triggerRefresh, 30_000);
    return () => clearInterval(interval);
  }, [triggerRefresh]);

  const [dark, setDark] = useState(() => {
    if (typeof window !== "undefined") {
      return (
        localStorage.getItem("theme") === "dark" ||
        (!localStorage.getItem("theme") &&
          window.matchMedia("(prefers-color-scheme: dark)").matches)
      );
    }
    return false;
  });

  const [bannerDismissed, setBannerDismissed] = useState(false);
  const [rulesOpen, setRulesOpen] = useState(false);

  // Reset banner dismissal when date range changes
  useEffect(() => {
    setBannerDismissed(false);
  }, [range.from, range.to]);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    localStorage.setItem("theme", dark ? "dark" : "light");
  }, [dark]);

  return (
    <DateRangeContext value={{ range, setRange, refreshKey, triggerRefresh }}>
      <div className="min-h-screen bg-background">
        <div className="max-w-screen-2xl mx-auto px-4 py-6">
          {/* Header */}
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 mb-6">
            <h1 className="text-2xl font-bold tracking-tight">Timely</h1>
            <div className="flex items-center gap-2">
              <DateRangePicker />
              <RulesPanel
                trigger={
                  <Button variant="ghost" size="icon" aria-label="Category rules">
                    <Settings className="h-4 w-4" />
                  </Button>
                }
                open={rulesOpen}
                onOpenChange={setRulesOpen}
              />
              <Button
                variant="ghost"
                size="icon"
                aria-label={dark ? "Switch to light mode" : "Switch to dark mode"}
                onClick={() => setDark(!dark)}
              >
                {dark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
              </Button>
            </div>
          </div>

          <DashboardContent
            bannerDismissed={bannerDismissed}
            onDismissBanner={() => setBannerDismissed(true)}
            onOpenRules={() => setRulesOpen(true)}
          />
        </div>
      </div>
    </DateRangeContext>
  );
}
