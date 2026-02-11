import { useState, useEffect } from "react";
import {
  startOfWeek,
  endOfWeek,
  startOfMonth,
  endOfMonth,
  startOfYear,
  endOfYear,
  subDays,
  format,
} from "date-fns";
import { CalendarIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { useDateRange } from "@/hooks/use-date-range";
import { toDateString } from "@/lib/format";
import type { DateRange as DateRangeType } from "@/lib/types";
import type { DateRange } from "react-day-picker";

const PRESETS: { label: string; getRange: () => Omit<DateRangeType, "label"> }[] = [
  {
    label: "Today",
    getRange: () => {
      const d = toDateString(new Date());
      return { from: d, to: d };
    },
  },
  {
    label: "Yesterday",
    getRange: () => {
      const d = toDateString(subDays(new Date(), 1));
      return { from: d, to: d };
    },
  },
  {
    label: "This Week",
    getRange: () => ({
      from: toDateString(startOfWeek(new Date(), { weekStartsOn: 1 })),
      to: toDateString(endOfWeek(new Date(), { weekStartsOn: 1 })),
    }),
  },
  {
    label: "This Month",
    getRange: () => ({
      from: toDateString(startOfMonth(new Date())),
      to: toDateString(endOfMonth(new Date())),
    }),
  },
  {
    label: "This Year",
    getRange: () => ({
      from: toDateString(startOfYear(new Date())),
      to: toDateString(endOfYear(new Date())),
    }),
  },
];

export function DateRangePicker() {
  const { range, setRange } = useDateRange();
  const [calendarOpen, setCalendarOpen] = useState(false);
  const [isMobile, setIsMobile] = useState(window.innerWidth < 640);

  useEffect(() => {
    const onResize = () => setIsMobile(window.innerWidth < 640);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  const handlePreset = (label: string, getRange: () => Omit<DateRangeType, "label">) => {
    const r = getRange();
    setRange({ ...r, label });
  };

  const handleCalendarSelect = (selected: DateRange | undefined) => {
    if (selected?.from) {
      const from = toDateString(selected.from);
      const to = selected.to ? toDateString(selected.to) : from;
      setRange({ from, to, label: "Custom" });
      if (selected.to) setCalendarOpen(false);
    }
  };

  return (
    <div className="flex items-center gap-2 flex-wrap">
      {PRESETS.map((preset) => (
        <Button
          key={preset.label}
          variant={range.label === preset.label ? "default" : "outline"}
          size="sm"
          onClick={() => handlePreset(preset.label, preset.getRange)}
        >
          {preset.label}
        </Button>
      ))}
      <Popover open={calendarOpen} onOpenChange={setCalendarOpen}>
        <PopoverTrigger asChild>
          <Button variant={range.label === "Custom" ? "default" : "outline"} size="sm">
            <CalendarIcon className="mr-2 h-4 w-4" />
            {range.label === "Custom"
              ? `${format(new Date(range.from + "T00:00"), "MMM d")} - ${format(new Date(range.to + "T00:00"), "MMM d")}`
              : "Custom"}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <Calendar
            mode="range"
            selected={{
              from: new Date(range.from + "T00:00"),
              to: new Date(range.to + "T00:00"),
            }}
            onSelect={handleCalendarSelect}
            numberOfMonths={isMobile ? 1 : 2}
          />
        </PopoverContent>
      </Popover>
    </div>
  );
}
