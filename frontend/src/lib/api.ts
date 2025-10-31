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

export async function uploadImage(
  args: {
    fileName: string;
    base64: string;
    mimeType?: string;
  },
  opts?: { onProgress?: (percent: number) => void }
): Promise<UploadResponse> {
  const body = JSON.stringify({ file_name: args.fileName, image_base64: args.base64, mime_type: args.mimeType });

  if (opts?.onProgress && typeof window !== "undefined") {
    return new Promise<UploadResponse>((resolve, reject) => {
      const xhr = new XMLHttpRequest();
      const href = apiPath("/api/images");

      xhr.open("POST", href);
      xhr.setRequestHeader("Content-Type", "application/json");

      xhr.upload.onprogress = (event) => {
        if (!event.lengthComputable) return;
        const percent = Math.round((event.loaded / event.total) * 100);
        opts.onProgress?.(percent);
      };

      xhr.onreadystatechange = () => {
        if (xhr.readyState !== XMLHttpRequest.DONE) return;

        const status = xhr.status;
        if (status < 200 || status >= 300) {
          reject(new Error(`Failed to upload: ${status}`));
          return;
        }

        try {
          const json = JSON.parse(xhr.responseText) as UploadResponse;
          resolve(json);
        } catch {
          reject(new Error("Upload succeeded but response was invalid JSON"));
        }
      };

      xhr.onerror = () => {
        reject(new Error("Failed to upload: network error"));
      };

      xhr.send(body);
    });
  }

  const href = typeof window === "undefined" ? `${API_BASE}/api/images` : apiPath("/api/images");
  const res = await fetch(href, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body,
  });
  if (!res.ok) throw new Error(`Failed to upload: ${res.status}`);
  return res.json();
}

export async function searchImages(query: string, opts?: { limit?: number; maxDistance?: number }): Promise<SearchResponse> {
  const res = await fetch(apiPath("/api/images/semantic-search"), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query, limit: opts?.limit, max_distance: opts?.maxDistance }),
  });
  if (!res.ok) throw new Error(`Failed to search: ${res.status}`);
  return res.json();
}
