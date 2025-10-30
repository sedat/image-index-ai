import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { Photo } from "@/lib/types";
import { API_BASE } from "@/lib/config";

function imageUrl(filePath: string) {
  // Try backend path first. If that fails at runtime due to CORS/static, consider proxying images.
  // We keep absolute URL for server rendering.
  const base = typeof window === "undefined" ? API_BASE : API_BASE;
  return `${base}/${filePath}`;
}

export function ImageCard({ photo }: { photo: Photo }) {
  return (
    <Card className="overflow-hidden">
      <div className="relative aspect-[4/3] w-full bg-zinc-100 dark:bg-zinc-900">
        {/* Use next/image with fallback to plain img if remote not allowed. */}
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={imageUrl(photo.file_path)}
          alt={photo.file_name}
          className="h-full w-full object-cover"
          loading="lazy"
        />
      </div>
      <div className="space-y-2 p-3">
        <div className="flex items-center justify-between gap-2">
          <div className="truncate text-sm font-medium" title={photo.file_name}>
            {photo.file_name}
          </div>
        </div>
        <div className="flex flex-wrap gap-1">
          {photo.tags.map((t, i) => (
            <Badge key={`${t}-${i}`}>{t}</Badge>
          ))}
        </div>
      </div>
    </Card>
  );
}
