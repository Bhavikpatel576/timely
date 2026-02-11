import { useState, useEffect, useMemo } from "react";
import { Trash2, ChevronDown, ChevronUp, Search } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
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
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { fetchApi, postApi, putApi, deleteApi } from "@/lib/api";
import { useDateRange } from "@/hooks/use-date-range";
import type { CategoryRule, Category } from "@/lib/types";

interface RulesPanelProps {
  trigger: React.ReactNode;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function RulesPanel({ trigger, open, onOpenChange }: RulesPanelProps) {
  const [rules, setRules] = useState<CategoryRule[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(false);
  const [newField, setNewField] = useState<"app" | "title" | "url_domain">("app");
  const [newPattern, setNewPattern] = useState("");
  const [newCategoryId, setNewCategoryId] = useState<string>("");
  const [confirmDeleteId, setConfirmDeleteId] = useState<number | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(
    () => new Set(["user-rules"])
  );
  const { triggerRefresh } = useDateRange();

  async function loadRules() {
    setLoading(true);
    try {
      const [rulesData, catsData] = await Promise.all([
        fetchApi<CategoryRule[]>("/api/rules"),
        fetchApi<Category[]>("/api/categories"),
      ]);
      setRules(rulesData);
      setCategories(catsData);
    } finally {
      setLoading(false);
    }
  }

  function handleOpenChange(isOpen: boolean) {
    if (isOpen) loadRules();
    setConfirmDeleteId(null);
    setSearchTerm("");
    onOpenChange?.(isOpen);
  }

  async function handleCategoryUpdate(ruleId: number, categoryId: string) {
    await putApi(`/api/rules/${ruleId}`, { category_id: Number(categoryId) });
    await loadRules();
    triggerRefresh();
  }

  async function handleDelete(ruleId: number) {
    await deleteApi(`/api/rules/${ruleId}`);
    setConfirmDeleteId(null);
    await loadRules();
    triggerRefresh();
  }

  async function handleAdd() {
    if (!newPattern.trim() || !newCategoryId) return;
    await postApi("/api/rules", {
      app: newPattern.trim(),
      category_id: Number(newCategoryId),
      field: newField,
    });
    setNewPattern("");
    setNewCategoryId("");
    await loadRules();
    triggerRefresh();
  }

  function toggleGroup(groupKey: string) {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(groupKey)) {
        next.delete(groupKey);
      } else {
        next.add(groupKey);
      }
      return next;
    });
  }

  // When controlled open changes, load rules
  useEffect(() => {
    if (open) loadRules();
  }, [open]);

  // Filter rules by search term
  const filteredRules = useMemo(() => {
    if (!searchTerm.trim()) return rules;
    const term = searchTerm.toLowerCase();
    return rules.filter(
      (rule) =>
        rule.pattern.toLowerCase().includes(term) ||
        rule.category_name.toLowerCase().includes(term) ||
        rule.field.toLowerCase().includes(term)
    );
  }, [rules, searchTerm]);

  // Split into user and built-in rules
  const userRules = useMemo(
    () => filteredRules.filter((r) => !r.is_builtin),
    [filteredRules]
  );
  const builtinRules = useMemo(
    () => filteredRules.filter((r) => r.is_builtin),
    [filteredRules]
  );

  // Group built-in rules by category name
  const builtinByCategory = useMemo(() => {
    const groups: Record<string, CategoryRule[]> = {};
    for (const rule of builtinRules) {
      const cat = rule.category_name;
      if (!groups[cat]) groups[cat] = [];
      groups[cat].push(rule);
    }
    // Sort category groups alphabetically
    return Object.entries(groups).sort(([a], [b]) => a.localeCompare(b));
  }, [builtinRules]);

  // When search is active, auto-expand groups that have matching results
  useEffect(() => {
    if (searchTerm.trim()) {
      const newExpanded = new Set<string>(["user-rules"]);
      newExpanded.add("builtin-rules");
      for (const [catName] of builtinByCategory) {
        newExpanded.add(`builtin-${catName}`);
      }
      setExpandedGroups(newExpanded);
    }
  }, [searchTerm, builtinByCategory]);

  function renderRuleRow(rule: CategoryRule) {
    return (
      <TableRow key={rule.id}>
        <TableCell className="font-mono text-sm">{rule.pattern}</TableCell>
        <TableCell className="text-sm text-muted-foreground">
          {rule.field}
        </TableCell>
        <TableCell>
          {rule.is_builtin ? (
            <span className="text-sm">{rule.category_name}</span>
          ) : (
            <Select
              value={String(rule.category_id)}
              onValueChange={(val) => handleCategoryUpdate(rule.id, val)}
            >
              <SelectTrigger size="sm" className="h-7 text-xs w-40">
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
          )}
        </TableCell>
        <TableCell className="text-right">
          {!rule.is_builtin &&
            (confirmDeleteId === rule.id ? (
              <div className="flex items-center justify-end gap-1">
                <Button
                  variant="destructive"
                  size="sm"
                  className="h-7 text-xs"
                  onClick={() => handleDelete(rule.id)}
                >
                  Confirm
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 text-xs"
                  onClick={() => setConfirmDeleteId(null)}
                >
                  Cancel
                </Button>
              </div>
            ) : (
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={() => setConfirmDeleteId(rule.id)}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </Button>
            ))}
        </TableCell>
      </TableRow>
    );
  }

  function renderRuleTable(rulesToRender: CategoryRule[]) {
    if (rulesToRender.length === 0) {
      return (
        <p className="text-sm text-muted-foreground py-2 px-1">
          No rules found.
        </p>
      );
    }
    return (
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Pattern</TableHead>
            <TableHead>Field</TableHead>
            <TableHead>Category</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>{rulesToRender.map(renderRuleRow)}</TableBody>
      </Table>
    );
  }

  function renderGroupHeader(
    groupKey: string,
    label: string,
    count: number,
    variant: "primary" | "secondary" = "secondary"
  ) {
    const isExpanded = expandedGroups.has(groupKey);
    return (
      <button
        className={`w-full flex items-center justify-between px-3 py-2 text-left rounded-md transition-colors ${
          variant === "primary"
            ? "bg-muted/50 hover:bg-muted"
            : "hover:bg-muted/30"
        }`}
        onClick={() => toggleGroup(groupKey)}
      >
        <div className="flex items-center gap-2">
          {isExpanded ? (
            <ChevronUp className="h-4 w-4 text-muted-foreground shrink-0" />
          ) : (
            <ChevronDown className="h-4 w-4 text-muted-foreground shrink-0" />
          )}
          <span
            className={`font-medium ${
              variant === "primary" ? "text-sm" : "text-xs"
            }`}
          >
            {label}
          </span>
        </div>
        <Badge variant="secondary" className="text-xs tabular-nums">
          {count}
        </Badge>
      </button>
    );
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>{trigger}</DialogTrigger>
      <DialogContent className="max-w-3xl max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>Category Rules</DialogTitle>
        </DialogHeader>

        {loading ? (
          <div className="space-y-3 py-4">
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="h-8 bg-muted animate-pulse rounded" />
            ))}
          </div>
        ) : (
          <div className="flex flex-col gap-4 min-h-0">
            {/* Search input */}
            <div className="relative">
              <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                placeholder="Search rules by pattern, category, or field..."
                className="h-9 pl-9 text-sm"
              />
            </div>

            {/* Add rule form */}
            <div className="border rounded-md p-3">
              <h4 className="text-sm font-medium mb-2">Add Rule</h4>
              <div className="flex items-end gap-2">
                <div className="flex-1">
                  <label className="text-xs text-muted-foreground mb-1 block">
                    Pattern
                  </label>
                  <Input
                    value={newPattern}
                    onChange={(e) => setNewPattern(e.target.value)}
                    placeholder="e.g. github.com"
                    className="h-8 text-sm"
                  />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground mb-1 block">
                    Field
                  </label>
                  <Select
                    value={newField}
                    onValueChange={(v) =>
                      setNewField(v as "app" | "title" | "url_domain")
                    }
                  >
                    <SelectTrigger className="h-8 text-xs w-28">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="app">app</SelectItem>
                      <SelectItem value="title">title</SelectItem>
                      <SelectItem value="url_domain">url_domain</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <label className="text-xs text-muted-foreground mb-1 block">
                    Category
                  </label>
                  <Select
                    value={newCategoryId}
                    onValueChange={setNewCategoryId}
                  >
                    <SelectTrigger className="h-8 text-xs w-40">
                      <SelectValue placeholder="Select..." />
                    </SelectTrigger>
                    <SelectContent>
                      {categories.map((cat) => (
                        <SelectItem key={cat.id} value={String(cat.id)}>
                          {cat.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <Button
                  size="sm"
                  className="h-8"
                  onClick={handleAdd}
                  disabled={!newPattern.trim() || !newCategoryId}
                >
                  Add
                </Button>
              </div>
            </div>

            {/* Scrollable rules list */}
            <div className="overflow-y-auto min-h-0 space-y-2 pr-1">
              {/* No results message */}
              {filteredRules.length === 0 && (
                <p className="text-sm text-muted-foreground text-center py-8">
                  No rules match &ldquo;{searchTerm}&rdquo;
                </p>
              )}

              {/* Your Rules section -- always shown first */}
              {(userRules.length > 0 || !searchTerm.trim()) && (
                <div className="space-y-1">
                  {renderGroupHeader(
                    "user-rules",
                    "Your Rules",
                    userRules.length,
                    "primary"
                  )}
                  {expandedGroups.has("user-rules") && (
                    <div className="pl-2">
                      {userRules.length > 0 ? (
                        renderRuleTable(userRules)
                      ) : (
                        <p className="text-sm text-muted-foreground py-3 px-1">
                          No custom rules yet. Add one above.
                        </p>
                      )}
                    </div>
                  )}
                </div>
              )}

              {/* Built-in Rules section */}
              {builtinRules.length > 0 && (
                <div className="space-y-1">
                  {renderGroupHeader(
                    "builtin-rules",
                    "Built-in Rules",
                    builtinRules.length,
                    "primary"
                  )}
                  {expandedGroups.has("builtin-rules") && (
                    <div className="pl-2 space-y-1">
                      {builtinByCategory.map(([catName, catRules]) => (
                        <div key={catName} className="space-y-1">
                          {renderGroupHeader(
                            `builtin-${catName}`,
                            catName,
                            catRules.length
                          )}
                          {expandedGroups.has(`builtin-${catName}`) && (
                            <div className="pl-4">
                              {renderRuleTable(catRules)}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
