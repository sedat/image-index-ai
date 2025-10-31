import { ImageCard } from "@/components/image-card";
import type { Photo } from "@/lib/types";

export function GalleryGrid({ photos }: { photos: Photo[] }) {
  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {photos.map((photo) => (
        <ImageCard key={photo.photo_id} photo={photo} />
      ))}
      {photos.length === 0 && (
        <div className="col-span-full text-sm text-zinc-500">No images found.</div>
      )}
    </div>
  );
}
