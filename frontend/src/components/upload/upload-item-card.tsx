import { StatusBadge } from "@/components/upload/status-badge";
import type { UploadItem } from "@/components/upload/use-uploads";
import { formatBytes } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "@/components/ui/card";

export function UploadItemCard({ item }: { item: UploadItem }) {
  return (
    <Card className="overflow-hidden">
      <CardHeader className="p-3">
        <div className="flex items-start justify-between gap-2">
          <div>
            <p className="break-words text-sm font-medium">{item.file.name}</p>
            <p className="text-xs text-muted-foreground">{formatBytes(item.file.size)}</p>
          </div>
          <StatusBadge status={item.status} />
        </div>
      </CardHeader>
      <CardContent className="p-3 pt-0">
        <div className="overflow-hidden rounded">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img src={item.preview} alt={item.file.name} className="h-32 w-full rounded object-cover" />
        </div>

        <div className="mt-3 h-2 w-full rounded-full bg-secondary">
          <div
            className="h-2 rounded-full bg-blue-500 transition-all"
            style={{ width: `${Math.min(item.progress, 100)}%` }}
          />
        </div>

        <div className="mt-2 flex items-center justify-between text-xs text-muted-foreground">
          <span>{item.progress}%</span>
          {item.error && <span className="text-red-500">{item.error}</span>}
        </div>
      </CardContent>
    </Card>
  );
}
