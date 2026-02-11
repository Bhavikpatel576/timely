import { useState } from "react";
import { ShieldAlert, ChevronDown, ChevronUp, ExternalLink } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useApi } from "@/hooks/use-api";
import { useDateRange } from "@/hooks/use-date-range";
import { fetchApi } from "@/lib/api";
import { formatDuration } from "@/lib/format";
import type { AppEntry, AppDetailsResponse } from "@/lib/types";

export function FlaggedActivitySection() {
  const { data: apps } = useApi<AppEntry[]>("/api/apps", { limit: "50" });
  const { range } = useDateRange();
  const [expanded, setExpanded] = useState<string | null>(null);
  const [sessions, setSessions] = useState<AppDetailsResponse | null>(null);
  const [sessionsLoading, setSessionsLoading] = useState(false);

  const flaggedApps = (apps || []).filter((a) => a.category === "inappropriate-content");

  if (flaggedApps.length === 0) return null;

  const totalSeconds = flaggedApps.reduce((sum, a) => sum + a.seconds, 0);

  async function toggleExpand(appName: string) {
    if (expanded === appName) {
      setExpanded(null);
      setSessions(null);
      return;
    }
    setExpanded(appName);
    setSessionsLoading(true);
    try {
      const result = await fetchApi<AppDetailsResponse>(
        `/api/apps/${encodeURIComponent(appName)}/details`,
        { from: range.from, to: range.to }
      );
      setSessions(result);
    } catch (err) {
      console.error("Failed to fetch sessions:", err);
    } finally {
      setSessionsLoading(false);
    }
  }

  return (
    <Card className="border-amber-200 dark:border-amber-800">
      <CardHeader>
        <div className="flex items-center gap-2">
          <ShieldAlert className="h-5 w-5 text-amber-500" />
          <CardTitle>Flagged Activity</CardTitle>
          <span className="ml-auto text-sm text-muted-foreground">
            {flaggedApps.length} {flaggedApps.length === 1 ? "site" : "sites"} · {formatDuration(totalSeconds)} total
          </span>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {flaggedApps.map((app) => {
          const isDomain = app.app.includes(".");
          const displayName = isDomain ? app.app.replace(/^www\./, "") : app.app;
          const isExpanded = expanded === app.app;

          return (
            <div key={app.app} className="rounded-lg border border-amber-100 dark:border-amber-900/50">
              <button
                className="w-full flex items-center justify-between px-4 py-3 text-left hover:bg-amber-50/50 dark:hover:bg-amber-950/20 rounded-lg transition-colors"
                onClick={() => toggleExpand(app.app)}
              >
                <div className="flex items-center gap-2 min-w-0">
                  <span className="font-medium truncate">{displayName}</span>
                  <span className="text-xs text-muted-foreground shrink-0">
                    {app.events} {app.events === 1 ? "visit" : "visits"}
                  </span>
                </div>
                <div className="flex items-center gap-3 shrink-0">
                  <span className="text-sm font-medium">{app.time}</span>
                  <span className="text-xs text-muted-foreground">{app.pct}%</span>
                  {isExpanded ? (
                    <ChevronUp className="h-4 w-4 text-muted-foreground" />
                  ) : (
                    <ChevronDown className="h-4 w-4 text-muted-foreground" />
                  )}
                </div>
              </button>

              {isExpanded && (
                <div className="px-4 pb-3 border-t border-amber-100 dark:border-amber-900/50">
                  {sessionsLoading ? (
                    <div className="space-y-2 py-2">
                      {Array.from({ length: 3 }).map((_, i) => (
                        <div key={i} className="h-5 bg-muted animate-pulse rounded" />
                      ))}
                    </div>
                  ) : sessions?.sessions?.length ? (
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead className="text-xs">Time</TableHead>
                          <TableHead className="text-xs">Page Title</TableHead>
                          <TableHead className="text-xs text-right">Duration</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {sessions.sessions.map((s, i) => {
                          const date = new Date(s.timestamp);
                          const time = date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
                          return (
                            <TableRow key={i}>
                              <TableCell className="text-xs text-muted-foreground whitespace-nowrap">{time}</TableCell>
                              <TableCell className="text-xs max-w-[250px] truncate">
                                {s.title || "—"}
                                {s.url && (
                                  <a
                                    href={s.url}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="ml-1 inline-flex items-center text-blue-500 hover:underline"
                                  >
                                    <ExternalLink className="h-3 w-3" />
                                  </a>
                                )}
                              </TableCell>
                              <TableCell className="text-xs text-right whitespace-nowrap">{formatDuration(s.duration)}</TableCell>
                            </TableRow>
                          );
                        })}
                      </TableBody>
                    </Table>
                  ) : (
                    <p className="text-sm text-muted-foreground py-2">No session details available</p>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
