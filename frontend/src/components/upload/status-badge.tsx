import type { UploadStatus } from "@/components/upload/use-uploads";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const STATUS_META: Record<UploadStatus, { label: string; className: string }> = {
  queued: {
    label: "Queued",
    className: "bg-zinc-200 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300",
  },
  encoding: {
    label: "Processing",
    className: "bg-blue-100 text-blue-700 dark:bg-blue-950 dark:text-blue-200",
  },
  uploading: {
    label: "Uploading",
    className: "bg-blue-100 text-blue-700 dark:bg-blue-950 dark:text-blue-200",
  },
  success: {
    label: "Done",
    className: "bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-200",
  },
  error: {
    label: "Failed",
    className: "bg-red-100 text-red-700 dark:bg-red-950 dark:text-red-200",
  },
};

export function StatusBadge({ status }: { status: UploadStatus }) {
  const meta = STATUS_META[status] ?? STATUS_META.queued;
  return <Badge className={cn("px-2", meta.className)}>{meta.label}</Badge>;
}
