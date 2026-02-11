import { useState, useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { useApi } from "@/hooks/use-api";
import { useDateRange } from "@/hooks/use-date-range";
import { postApi } from "@/lib/api";
import type { TimelineEntry, Category } from "@/lib/types";
import { getCategoryColor } from "@/lib/colors";
import { formatDuration } from "@/lib/format";
import { isInappropriate } from "@/lib/inappropriate";

const HOURS = Array.from({ length: 25 }, (_, i) => i);

export function TimelineBlocks() {
  const { data, loading } = useApi<TimelineEntry[]>("/api/timeline", { limit: "500" });
  const { data: categories } = useApi<Category[]>("/api/categories", undefined, {
    includeDateRange: false,
  });
  const { triggerRefresh } = useDateRange();
  const [activeBlockId, setActiveBlockId] = useState<number | null>(null);

  const blocks = useMemo(() => {
    if (!data || data.length === 0) return [];

    return data.map((entry, idx) => {
      const date = new Date(entry.timestamp);
      const startHour = date.getHours() + date.getMinutes() / 60;
      const durationHours = entry.duration / 3600;

      return {
        ...entry,
        id: idx,
        startHour,
        durationHours: Math.max(durationHours, 0.05), // min visible width
        color: getCategoryColor(entry.category),
        flagged: isInappropriate(entry.category),
        startTime: date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
      };
    });
  }, [data]);

  async function handleCategoryChange(app: string, categoryId: string) {
    const field = app.includes(".") ? "url_domain" : "app";
    try {
      await postApi("/api/rules", {
        app,
        category_id: Number(categoryId),
        field,
      });
      triggerRefresh();
    } catch (err) {
      console.error("Failed to update category:", err);
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Activity Timeline</CardTitle>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="h-16 bg-muted animate-pulse rounded" />
        ) : blocks.length === 0 ? (
          <div className="py-8 text-center text-muted-foreground">
            No data for this period
          </div>
        ) : (
          <div>
            <div className="overflow-hidden">
              {/* Hour ruler */}
              <div className="relative h-5 text-xs text-muted-foreground mb-1">
                {HOURS.filter((h) => h % 3 === 0).map((h) => (
                  <span
                    key={h}
                    className="absolute"
                    style={{
                      left: `${(h / 24) * 100}%`,
                      transform: "translateX(-50%)",
                    }}
                  >
                    {h.toString().padStart(2, "0")}
                  </span>
                ))}
              </div>

              {/* Timeline bar */}
              <div className="relative h-10 bg-muted rounded overflow-hidden">
              {blocks.map((block) => (
                <Popover
                  key={block.id}
                  open={activeBlockId === block.id}
                  onOpenChange={(open) => setActiveBlockId(open ? block.id : null)}
                >
                  <PopoverTrigger asChild>
                    <div
                      className={
                        "absolute top-0 h-full opacity-90 hover:opacity-100 transition-opacity cursor-pointer" +
                        (block.flagged ? " flagged-timeline-block" : "")
                      }
                      style={{
                        left: `${(block.startHour / 24) * 100}%`,
                        width: `${Math.max((block.durationHours / 24) * 100, 0.2)}%`,
                        backgroundColor: block.color,
                      }}
                    />
                  </PopoverTrigger>
                  <PopoverContent className="w-80 text-sm space-y-2" side="top">
                    <div className="flex items-center justify-between">
                      <span className="font-medium truncate">
                        {block.app || "Unknown"}
                      </span>
                      <Badge
                        variant="outline"
                        style={{
                          borderColor: getCategoryColor(block.category),
                          color: getCategoryColor(block.category),
                        }}
                      >
                        {isInappropriate(block.category) ? "Flagged" : block.category}
                      </Badge>
                    </div>
                    {block.title && (
                      <p className="text-muted-foreground truncate text-xs">{block.title}</p>
                    )}
                    {block.url && (
                      <a
                        href={block.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-xs text-blue-500 hover:underline truncate block"
                      >
                        {block.url}
                      </a>
                    )}
                    <div className="flex items-center justify-between text-xs text-muted-foreground">
                      <span>{block.startTime}</span>
                      <span>{formatDuration(block.duration)}</span>
                    </div>
                    {block.category === "uncategorized" && categories && block.app && (
                      <Select
                        onValueChange={(val) => handleCategoryChange(block.app!, val)}
                      >
                        <SelectTrigger size="sm" className="h-7 text-xs">
                          <SelectValue placeholder="Assign category..." />
                        </SelectTrigger>
                        <SelectContent>
                          {categories.map((cat) => (
                            <SelectItem key={cat.id} value={String(cat.id)}>
                              {cat.name}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    )}
                  </PopoverContent>
                </Popover>
              ))}
            </div>
            </div>

            {/* Legend */}
            <div className="flex flex-wrap gap-3 mt-3">
              {Array.from(new Set(blocks.map((b) => b.category))).map((cat) => (
                <div key={cat} className="flex items-center gap-1.5 text-xs">
                  <div
                    className="w-3 h-3 rounded-sm"
                    style={{ backgroundColor: getCategoryColor(cat) }}
                  />
                  <span className="text-muted-foreground">
                    {isInappropriate(cat) ? "Flagged" : cat}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
