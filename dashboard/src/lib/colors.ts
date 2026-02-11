const CATEGORY_COLORS: Record<string, string> = {
  "work/coding": "#3b82f6",       // blue
  "work/communication": "#06b6d4", // cyan
  "work/documentation": "#8b5cf6", // violet
  "work/devops": "#6366f1",        // indigo
  "entertainment/video": "#ef4444", // red
  "entertainment/social": "#f97316", // orange
  "entertainment/music": "#ec4899", // pink
  "productivity/reading": "#10b981", // emerald
  "productivity/finance": "#14b8a6", // teal
  "uncategorized": "#9ca3af",       // gray
  "inappropriate-content": "#78716c", // stone
};

const FALLBACK_COLORS = [
  "#0ea5e9", "#a855f7", "#f59e0b", "#22c55e", "#e11d48",
  "#64748b", "#84cc16", "#d946ef", "#0891b2", "#dc2626",
];

function hashCode(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) - hash) + str.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

export function getCategoryColor(category: string): string {
  if (CATEGORY_COLORS[category]) return CATEGORY_COLORS[category];

  // Check parent match (e.g., "work" matches "work/coding")
  for (const [key, color] of Object.entries(CATEGORY_COLORS)) {
    if (key.startsWith(category + "/") || category.startsWith(key.split("/")[0] + "/")) {
      return color;
    }
  }

  // Assign a deterministic fallback based on category name hash
  if (!CATEGORY_COLORS[category]) {
    CATEGORY_COLORS[category] = FALLBACK_COLORS[hashCode(category) % FALLBACK_COLORS.length];
  }

  return CATEGORY_COLORS[category];
}

export function getProductivityColor(score: number): string {
  if (score >= 70) return "#10b981"; // green
  if (score >= 50) return "#f59e0b"; // amber
  return "#ef4444"; // red
}
