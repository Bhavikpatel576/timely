import { ShieldAlert, X } from "lucide-react";
import { Button } from "@/components/ui/button";

interface FlaggedContentBannerProps {
  onDismiss: () => void;
  onManageRules: () => void;
}

export function FlaggedContentBanner({
  onDismiss,
  onManageRules,
}: FlaggedContentBannerProps) {
  return (
    <div className="flex items-center gap-3 rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 dark:border-amber-700 dark:bg-amber-950/50">
      <ShieldAlert className="h-5 w-5 shrink-0 text-amber-600 dark:text-amber-400" />
      <span className="flex-1 text-sm text-amber-800 dark:text-amber-200">
        Activity flagged as inappropriate content was detected
      </span>
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          className="h-7 text-xs text-amber-700 hover:text-amber-900 dark:text-amber-300 dark:hover:text-amber-100"
          onClick={() =>
            document
              .getElementById("app-breakdown-table")
              ?.scrollIntoView({ behavior: "smooth" })
          }
        >
          Review in table
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="h-7 text-xs text-amber-700 hover:text-amber-900 dark:text-amber-300 dark:hover:text-amber-100"
          onClick={onManageRules}
        >
          Manage Rules
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7 text-amber-600 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200"
          onClick={onDismiss}
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
