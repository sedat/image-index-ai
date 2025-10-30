import { API_BASE } from "@/lib/config";
import type { PhotosResponse, SearchResponse, UploadResponse } from "@/lib/types";

function apiPath(path: string) {
  if (typeof window === "undefined") return `${API_BASE}${path}`;
  const clientPath = path.startsWith("/api") ? path.slice(4) : path;
  return `/api/proxy${clientPath}`;
}

export async function listImages(tags?: string[]): Promise<PhotosResponse> {
  const url = new URL(apiPath("/api/images"), "http://localhost");
  if (tags && tags.length) url.searchParams.set("tags", tags.join(","));
  const href = typeof window === "undefined" ? `${API_BASE}/api/images${url.search}` : url.pathname + url.search;
  const res = await fetch(href, { next: { revalidate: 0 } });
  if (!res.ok) throw new Error(`Failed to list images: ${res.status}`);
  return res.json();
}

export async function uploadImage(args: {
  fileName: string;
  base64: string;
  mimeType?: string;
}): Promise<UploadResponse> {
  const res = await fetch(apiPath("/api/images"), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ file_name: args.fileName, image_base64: args.base64, mime_type: args.mimeType }),
  });
  if (!res.ok) throw new Error(`Failed to upload: ${res.status}`);
  return res.json();
}

export async function searchImages(query: string): Promise<SearchResponse> {
  const res = await fetch(apiPath("/api/images/search"), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });
  if (!res.ok) throw new Error(`Failed to search: ${res.status}`);
  return res.json();
}
