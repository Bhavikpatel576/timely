import { useState, useMemo } from "react";
import { ShieldAlert, Eye, EyeOff, Globe, Monitor } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useApi } from "@/hooks/use-api";
import { useDateRange } from "@/hooks/use-date-range";
import { postApi, fetchApi } from "@/lib/api";
import { getCategoryColor } from "@/lib/colors";
import { isInappropriate } from "@/lib/inappropriate";
import { formatDuration } from "@/lib/format";
import type { AppEntry, Category, AppDetailsResponse } from "@/lib/types";

export function AppBreakdownTable() {
  const { data, loading } = useApi<AppEntry[]>("/api/apps", { limit: "20" });
  const { data: categories } = useApi<Category[]>("/api/categories", undefined, {
    includeDateRange: false,
  });
  const { range, triggerRefresh } = useDateRange();
  const [updatingApp, setUpdatingApp] = useState<string | null>(null);
  const [privacyMode, setPrivacyMode] = useState(false);
  const [detailApp, setDetailApp] = useState<string | null>(null);
  const [detailData, setDetailData] = useState<AppDetailsResponse | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);

  const sortedData = useMemo(() => {
    if (!data) return [];
    return [...data].sort((a, b) => {
      const aFlagged = isInappropriate(a.category) ? 1 : 0;
      const bFlagged = isInappropriate(b.category) ? 1 : 0;
      return bFlagged - aFlagged;
    });
  }, [data]);

  function inferField(name: string): "app" | "url_domain" {
    return name.includes(".") ? "url_domain" : "app";
  }

  async function handleCategoryChange(app: string, categoryId: string) {
    setUpdatingApp(app);
    try {
      await postApi("/api/rules", {
        app,
        category_id: Number(categoryId),
        field: inferField(app),
      });
      triggerRefresh();
    } catch (err) {
      console.error("Failed to update category:", err);
    } finally {
      setUpdatingApp(null);
    }
  }

  function getCategoryId(categoryName: string): string | undefined {
    const cat = categories?.find((c) => c.name === categoryName);
    return cat ? String(cat.id) : undefined;
  }

  async function openDetail(appName: string) {
    setDetailApp(appName);
    setDetailLoading(true);
    try {
      const result = await fetchApi<AppDetailsResponse>(
        `/api/apps/${encodeURIComponent(appName)}/details`,
        { from: range.from, to: range.to }
      );
      setDetailData(result);
    } catch (err) {
      console.error("Failed to fetch app details:", err);
    } finally {
      setDetailLoading(false);
    }
  }

  return (
    <>
      <Card id="app-breakdown-table">
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Top Apps & Sites</CardTitle>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={() => setPrivacyMode(!privacyMode)}
              title={privacyMode ? "Show flagged names" : "Hide flagged names"}
            >
              {privacyMode ? (
                <EyeOff className="h-4 w-4" />
              ) : (
                <Eye className="h-4 w-4" />
              )}
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="space-y-3">
              {Array.from({ length: 5 }).map((_, i) => (
                <div key={i} className="h-8 bg-muted animate-pulse rounded" />
              ))}
            </div>
          ) : !sortedData || sortedData.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              No data for this period
            </div>
          ) : (
            <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>App</TableHead>
                  <TableHead>Category</TableHead>
                  <TableHead className="text-right">Time</TableHead>
                  <TableHead className="text-right">%</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedData.map((app) => {
                  const flagged = isInappropriate(app.category);
                  const isDomain = app.app.includes(".");
                  const displayName = isDomain ? app.app.replace(/^www\./, "") : app.app;
                  const isUncategorized = app.category === "uncategorized";
                  return (
                    <TableRow
                      key={app.app}
                      className={
                        "cursor-pointer hover:bg-muted/50 " +
                        (updatingApp === app.app ? "opacity-50 " : "") +
                        (flagged ? "border-l-2 border-l-amber-500 " : "") +
                        (isUncategorized ? "bg-amber-50 dark:bg-amber-950/20" : "")
                      }
                      onClick={() => openDetail(app.app)}
                    >
                      <TableCell className="font-medium">
                        <div className="flex items-center gap-1.5">
                          {flagged && (
                            <ShieldAlert className="h-4 w-4 shrink-0 text-amber-500" />
                          )}
                          {!flagged && isDomain && (
                            <Globe className="h-4 w-4 shrink-0 text-muted-foreground" />
                          )}
                          {!flagged && !isDomain && (
                            <Monitor className="h-4 w-4 shrink-0 text-muted-foreground" />
                          )}
                          {flagged && privacyMode ? (
                            <span className="italic text-muted-foreground">
                              Flagged content
                            </span>
                          ) : (
                            displayName
                          )}
                        </div>
                      </TableCell>
                      <TableCell onClick={(e) => e.stopPropagation()}>
                        {categories ? (
                          <Select
                            value={getCategoryId(app.category)}
                            onValueChange={(val) => handleCategoryChange(app.app, val)}
                            disabled={updatingApp === app.app}
                          >
                            <SelectTrigger
                              size="sm"
                              className="h-7 text-xs border"
                              style={{
                                borderColor: getCategoryColor(app.category),
                                color: getCategoryColor(app.category),
                              }}
                            >
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {categories.map((cat) => (
                                <SelectItem key={cat.id} value={String(cat.id)}>
                                  {cat.name}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        ) : (
                          <span
                            className="text-xs"
                            style={{ color: getCategoryColor(app.category) }}
                          >
                            {app.category}
                          </span>
                        )}
                      </TableCell>
                      <TableCell className="text-right">{app.time}</TableCell>
                      <TableCell className="text-right">{app.pct}%</TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
            </div>
          )}
        </CardContent>
      </Card>

      {/* App drill-down dialog */}
      <Dialog open={detailApp !== null} onOpenChange={(open) => { if (!open) { setDetailApp(null); setDetailData(null); } }}>
        <DialogContent className="sm:max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{detailApp}</DialogTitle>
            <DialogDescription>Session details for this period</DialogDescription>
          </DialogHeader>
          {detailLoading ? (
            <div className="space-y-2">
              {Array.from({ length: 5 }).map((_, i) => (
                <div key={i} className="h-6 bg-muted animate-pulse rounded" />
              ))}
            </div>
          ) : detailData?.sessions?.length ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Time</TableHead>
                  <TableHead>Title</TableHead>
                  <TableHead className="text-right">Duration</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {detailData.sessions.map((session, i) => {
                  const date = new Date(session.timestamp);
                  const time = date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
                  return (
                    <TableRow key={i}>
                      <TableCell className="text-muted-foreground whitespace-nowrap">{time}</TableCell>
                      <TableCell className="max-w-[300px] truncate">
                        {session.title || "—"}
                        {session.url && (
                          <a
                            href={session.url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="block text-xs text-blue-500 hover:underline truncate"
                          >
                            {session.url}
                          </a>
                        )}
                      </TableCell>
                      <TableCell className="text-right whitespace-nowrap">{formatDuration(session.duration)}</TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          ) : (
            <div className="py-4 text-center text-muted-foreground">No sessions found</div>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}
