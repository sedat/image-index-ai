import { Suspense } from "react";
import { SearchBar } from "@/components/gallery/search-bar";
import { GalleryGrid } from "@/components/gallery/gallery-grid";
import { listImages, searchImages } from "@/lib/api";

type HomeSearchParams = {
  q?: string | string[];
  tags?: string | string[];
};

export default async function Home({ searchParams }: { searchParams: Promise<HomeSearchParams> }) {
  const sp = await searchParams;
  const qParam = Array.isArray(sp.q) ? sp.q[0] : sp.q;
  const tagsParam = Array.isArray(sp.tags) ? sp.tags[0] : sp.tags;
  const query = qParam?.trim();
  const tags = tagsParam?.split(",").filter(Boolean);
  const data = query ? await searchImages(query, { limit: 10, maxDistance: 0.4 }) : await listImages(tags);
  const photos = data.photos;

  return (
    <div className="space-y-6">
      <SearchBar defaultQuery={query ?? ""} />

      <Suspense>
        <GalleryGrid photos={photos} />
      </Suspense>
    </div>
  );
}
