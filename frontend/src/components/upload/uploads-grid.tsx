import { UploadItemCard } from "@/components/upload/upload-item-card";
import type { UploadItem } from "@/components/upload/use-uploads";

export function UploadsGrid({ items }: { items: UploadItem[] }) {
  if (!items.length) return null;

  return (
    <div className="grid gap-3 md:grid-cols-2">
      {items.map((item) => (
        <UploadItemCard key={item.id} item={item} />
      ))}
    </div>
  );
}
