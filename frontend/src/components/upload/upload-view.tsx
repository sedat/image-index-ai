"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { FilePicker } from "@/components/upload/file-picker";
import { UploadSummary } from "@/components/upload/upload-summary";
import { UploadsGrid } from "@/components/upload/uploads-grid";
import { useUploads } from "@/components/upload/use-uploads";
import { useRouter } from "next/navigation";

export function UploadView() {
  const router = useRouter();
  const { items, isUploading, message, summary, handleFileInputChange, handleSubmit, retryFailed } = useUploads({
    onSuccess: () => router.push("/"),
  });

  const hasFailedItems = items.some((item) => item.status === "error");

  return (
    <div className="mx-auto max-w-3xl">
      <Card>
        <CardHeader>
          <CardTitle>Upload Images</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <FilePicker disabled={isUploading} onChange={handleFileInputChange} />

            {summary.total > 0 && (
              <div className="space-y-3">
                <UploadSummary {...summary} />
                <UploadsGrid items={items} />
              </div>
            )}

            <div className="flex flex-wrap items-center gap-2">
              <Button type="submit" disabled={summary.total === 0 || isUploading}>
                {isUploading ? "Uploading..." : summary.total > 1 ? `Upload ${summary.total} images` : "Upload"}
              </Button>
              <Button type="button" variant="outline" disabled={isUploading || !hasFailedItems} onClick={retryFailed}>
                Retry failed
              </Button>
              {message && <p className="text-sm text-zinc-600 dark:text-zinc-400">{message}</p>}
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
