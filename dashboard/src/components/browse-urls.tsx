import { useState, useEffect, useMemo } from "react";
import { Search, ExternalLink, ChevronLeft, ChevronRight } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useApi } from "@/hooks/use-api";
import { formatDuration } from "@/lib/format";
import type { UrlsResponse, Category } from "@/lib/types";

export function BrowseUrls() {
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [domain, setDomain] = useState("");
  const [categoryId, setCategoryId] = useState("");
  const [page, setPage] = useState(1);
  const [sort, setSort] = useState("timestamp");
  const [order, setOrder] = useState("desc");
  const limit = 50;

  // Debounce search input
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearch(search);
      setPage(1);
    }, 300);
    return () => clearTimeout(timer);
  }, [search]);

  // Reset page when filters change
  useEffect(() => {
    setPage(1);
  }, [domain, categoryId]);

  const params = useMemo(() => {
    const p: Record<string, string> = {
      page: String(page),
      limit: String(limit),
      sort,
      order,
    };
    if (debouncedSearch) p.search = debouncedSearch;
    if (domain) p.domain = domain;
    if (categoryId) p.category = categoryId;
    return p;
  }, [page, debouncedSearch, domain, categoryId, sort, order]);

  const { data, loading } = useApi<UrlsResponse>("/api/urls", params);
  const { data: categories } = useApi<Category[]>("/api/categories", undefined, {
    includeDateRange: false,
  });

  const totalPages = data ? Math.ceil(data.total / limit) : 0;
  const showingFrom = data && data.total > 0 ? (page - 1) * limit + 1 : 0;
  const showingTo = data ? Math.min(page * limit, data.total) : 0;

  function handleSort(col: string) {
    if (sort === col) {
      setOrder(order === "asc" ? "desc" : "asc");
    } else {
      setSort(col);
      setOrder("desc");
    }
    setPage(1);
  }

  function sortIndicator(col: string) {
    if (sort !== col) return null;
    return order === "asc" ? " \u2191" : " \u2193";
  }

  function formatTimestamp(ts: string) {
    const d = new Date(ts);
    return d.toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function truncate(str: string | null, max: number) {
    if (!str) return "\u2014";
    return str.length > max ? str.slice(0, max) + "..." : str;
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Browser URL History</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Filter bar */}
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search URLs, domains, page titles..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-9"
            />
          </div>
          <Select value={domain} onValueChange={(v) => setDomain(v === "all" ? "" : v)}>
            <SelectTrigger className="w-full sm:w-[200px]">
              <SelectValue placeholder="All domains" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All domains</SelectItem>
              {data?.domains?.map((d) => (
                <SelectItem key={d} value={d}>
                  {d}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={categoryId} onValueChange={(v) => setCategoryId(v === "all" ? "" : v)}>
            <SelectTrigger className="w-full sm:w-[200px]">
              <SelectValue placeholder="All categories" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All categories</SelectItem>
              {categories?.map((c) => (
                <SelectItem key={c.id} value={String(c.id)}>
                  {c.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Results table */}
        {loading && !data ? (
          <div className="text-center py-12 text-muted-foreground">Loading...</div>
        ) : !data || data.rows.length === 0 ? (
          <div className="text-center py-12 text-muted-foreground">
            No browser URLs found for this date range.
          </div>
        ) : (
          <>
            <div className="rounded-md border overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="cursor-pointer select-none" onClick={() => handleSort("url_domain")}>
                      Domain{sortIndicator("url_domain")}
                    </TableHead>
                    <TableHead>Page Title</TableHead>
                    <TableHead>URL</TableHead>
                    <TableHead className="cursor-pointer select-none" onClick={() => handleSort("timestamp")}>
                      Time{sortIndicator("timestamp")}
                    </TableHead>
                    <TableHead className="cursor-pointer select-none" onClick={() => handleSort("duration")}>
                      Duration{sortIndicator("duration")}
                    </TableHead>
                    <TableHead className="cursor-pointer select-none" onClick={() => handleSort("category")}>
                      Category{sortIndicator("category")}
                    </TableHead>
                    <TableHead className="cursor-pointer select-none" onClick={() => handleSort("app")}>
                      App{sortIndicator("app")}
                    </TableHead>
                    <TableHead>AFK</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {data.rows.map((row, i) => (
                    <TableRow key={`${row.timestamp}-${i}`}>
                      <TableCell className="font-medium whitespace-nowrap">
                        {row.url_domain || "\u2014"}
                      </TableCell>
                      <TableCell className="max-w-[200px]" title={row.title || undefined}>
                        {truncate(row.title, 40)}
                      </TableCell>
                      <TableCell className="max-w-[250px]">
                        <a
                          href={row.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-blue-600 dark:text-blue-400 hover:underline inline-flex items-center gap-1"
                          title={row.url}
                        >
                          {truncate(row.url, 50)}
                          <ExternalLink className="h-3 w-3 shrink-0" />
                        </a>
                      </TableCell>
                      <TableCell className="whitespace-nowrap">
                        {formatTimestamp(row.timestamp)}
                      </TableCell>
                      <TableCell className="whitespace-nowrap">
                        {formatDuration(row.duration)}
                      </TableCell>
                      <TableCell>
                        <Badge variant="secondary">{row.category}</Badge>
                      </TableCell>
                      <TableCell className="whitespace-nowrap">{row.app || "\u2014"}</TableCell>
                      <TableCell>
                        {row.is_afk ? (
                          <Badge variant="outline" className="text-yellow-600 border-yellow-300">
                            AFK
                          </Badge>
                        ) : null}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>

            {/* Pagination */}
            <div className="flex items-center justify-between">
              <div className="text-sm text-muted-foreground">
                Showing {showingFrom}–{showingTo} of {data.total} URLs
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={page <= 1}
                  onClick={() => setPage((p) => p - 1)}
                >
                  <ChevronLeft className="h-4 w-4" />
                  Prev
                </Button>
                <span className="text-sm">
                  Page {page} of {totalPages}
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={page >= totalPages}
                  onClick={() => setPage((p) => p + 1)}
                >
                  Next
                  <ChevronRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
