"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { uploadImage } from "@/lib/api";
import { useRouter } from "next/router";

export default function UploadPage() {
  const [file, setFile] = useState<File | null>(null);
  const [preview, setPreview] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const router = useRouter();

  const onChange: React.ChangeEventHandler<HTMLInputElement> = async (e) => {
    const f = e.target.files?.[0] ?? null;
    setFile(f);
    setMessage(null);
    if (!f) return setPreview(null);
    const url = URL.createObjectURL(f);
    setPreview(url);
  };

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!file) return;
    setIsUploading(true);
    setMessage(null);
    try {
      const base64 = await toBase64(file);
      const res = await uploadImage({ fileName: file.name, base64, mimeType: file.type || undefined });
      setMessage(`Uploaded: ${res.photo.file_name}`);
      setFile(null);
      setPreview(null);
      router.push(`/`);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Upload failed";
      setMessage(message);
    } finally {
      setIsUploading(false);
      
    }
  }

  return (
    <div className="mx-auto max-w-xl">
      <Card>
        <CardHeader>
          <CardTitle>Upload Image</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="file">Choose image</Label>
              <Input id="file" type="file" accept="image/*" onChange={onChange} />
            </div>
            {preview && (
              <div className="overflow-hidden rounded-md border border-zinc-200 dark:border-zinc-800">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img src={preview} alt="preview" className="max-h-80 w-full object-contain bg-zinc-50 dark:bg-zinc-900" />
              </div>
            )}
            <div className="flex items-center gap-2">
              <Button type="submit" disabled={!file || isUploading}>
                {isUploading ? "Uploading..." : "Upload"}
              </Button>
              {message && <p className="text-sm text-zinc-600 dark:text-zinc-400">{message}</p>}
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

async function toBase64(file: File): Promise<string> {
  const buf = await file.arrayBuffer();
  let binary = "";
  const bytes = new Uint8Array(buf);
  const len = bytes.byteLength;
  for (let i = 0; i < len; i++) binary += String.fromCharCode(bytes[i]);
  return btoa(binary);
}
