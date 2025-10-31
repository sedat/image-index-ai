export type Photo = {
  photo_id: number;
  file_name: string;
  file_path: string; // relative to backend root, e.g. images/foo.jpg
  tags: string[];
  created_at: string; // ISO string
};

export type PhotosResponse = { photos: Photo[] };
export type UploadResponse = { photo: Photo };
export type SearchResponse = { query: string; photos: Photo[]; tags?: string[] };
