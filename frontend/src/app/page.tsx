import Link from "next/link";
import { listImages, searchImages } from "@/lib/api";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { ImageCard } from "@/components/image-card";
import { Suspense } from "react";

export default async function Home({ searchParams }: { searchParams: Promise<{ q?: string | string[]; tags?: string | string[] }> }) {
  const sp = await searchParams;
  const qParam = Array.isArray(sp.q) ? sp.q[0] : sp.q;
  const tagsParam = Array.isArray(sp.tags) ? sp.tags[0] : sp.tags;
  const query = qParam?.trim();
  const tags = tagsParam?.split(",").filter(Boolean);
  const data = query ? await searchImages(query) : await listImages(tags);
  const photos = query ? data.photos : data.photos;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-2">
        <h1 className="text-xl font-semibold">Gallery</h1>
        <Button asChild>
          <Link href="/upload">Upload</Link>
        </Button>
      </div>

      <form action="/" method="get" className="flex items-center gap-2">
        <Input
          name="q"
          placeholder="Search by description or tags..."
          defaultValue={query ?? ""}
        />
        <Button type="submit">Search</Button>
      </form>

      <Suspense>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {photos.map((p) => (
            <ImageCard key={p.photo_id} photo={p} />
          ))}
          {photos.length === 0 && (
            <div className="col-span-full text-sm text-zinc-500">No images found.</div>
          )}
        </div>
      </Suspense>
    </div>
  );
}
