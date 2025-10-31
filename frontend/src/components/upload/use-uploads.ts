"use client";

import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ChangeEventHandler,
  type FormEvent,
} from "react";
import { uploadImage } from "@/lib/api";

const MAX_CONCURRENCY = 4;

export type UploadStatus = "queued" | "encoding" | "uploading" | "success" | "error";

export type UploadItem = {
  id: string;
  file: File;
  preview: string;
  progress: number;
  status: UploadStatus;
  error?: string;
};

type UseUploadsOptions = {
  onSuccess?: () => void;
};

export function useUploads(options?: UseUploadsOptions) {
  const [items, setItems] = useState<UploadItem[]>([]);
  const [isUploading, setIsUploading] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    return () => {
      items.forEach((item) => URL.revokeObjectURL(item.preview));
    };
  }, [items]);

  const updateItem = useCallback((id: string, patch: Partial<UploadItem>) => {
    setItems((prev) =>
      prev.map((item) => {
        if (item.id !== id) return item;
        return { ...item, ...patch };
      })
    );
  }, []);

  const readFileAsBase64 = useCallback(
    (item: UploadItem) => {
      return new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onerror = () => {
          reject(reader.error ?? new Error("Failed to read file"));
        };
        reader.onprogress = (event) => {
          if (!event.lengthComputable) return;
          const fraction = event.loaded / event.total;
          const progress = Math.max(1, Math.min(60, Math.round(fraction * 60)));
          updateItem(item.id, { progress, status: "encoding", error: undefined });
        };
        reader.onload = () => {
          const buffer = reader.result as ArrayBuffer;
          const bytes = new Uint8Array(buffer);
          let binary = "";
          const chunk = 0x8000;
          for (let i = 0; i < bytes.length; i += chunk) {
            const sub = bytes.subarray(i, i + chunk);
            binary += String.fromCharCode(...sub);
          }
          resolve(btoa(binary));
        };
        reader.readAsArrayBuffer(item.file);
      });
    },
    [updateItem]
  );

  const uploadSingle = useCallback(
    async (item: UploadItem) => {
      try {
        updateItem(item.id, { status: "encoding", progress: Math.max(item.progress, 1), error: undefined });
        const base64 = await readFileAsBase64(item);

        updateItem(item.id, { status: "uploading", progress: Math.max(60, item.progress) });

        await uploadImage(
          {
            fileName: item.file.name,
            base64,
            mimeType: item.file.type || undefined,
          },
          {
            onProgress: (percent) => {
              const uploadProgress = 60 + Math.round((percent / 100) * 39);
              updateItem(item.id, {
                progress: Math.max(uploadProgress, 61),
                status: "uploading",
              });
            },
          }
        );

        updateItem(item.id, { status: "success", progress: 100, error: undefined });
        return true;
      } catch (error) {
        const err = error instanceof Error ? error : new Error("Upload failed");
        updateItem(item.id, { status: "error", progress: 100, error: err.message });
        return false;
      }
    },
    [readFileAsBase64, updateItem]
  );

  const processUploads = useCallback(
    async (targets: UploadItem[]) => {
      if (!targets.length) return { hasError: false };
      let hasError = false;
      let nextIndex = 0;

      const workerCount = Math.min(MAX_CONCURRENCY, targets.length);

      const workers = Array.from({ length: workerCount }, async () => {
        while (true) {
          const currentIndex = nextIndex++;
          if (currentIndex >= targets.length) break;
          const currentItem = targets[currentIndex];
          const ok = await uploadSingle(currentItem);
          if (!ok) hasError = true;
        }
      });

      await Promise.all(workers);
      return { hasError };
    },
    [uploadSingle]
  );

  const handleFileInputChange = useCallback<ChangeEventHandler<HTMLInputElement>>(
    (event) => {
      const files = Array.from(event.target.files ?? []);
      if (!files.length) {
        setItems([]);
        setMessage(null);
        event.target.value = "";
        return;
      }

      const nextItems = files.map((file) => ({
        id:
          typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
            ? crypto.randomUUID()
            : `${file.name}-${file.size}-${file.lastModified}-${Math.random().toString(36).slice(2)}`,
        file,
        preview: URL.createObjectURL(file),
        progress: 0,
        status: "queued" as UploadStatus,
        error: undefined,
      }));

      setMessage(null);
      setItems(nextItems);
      event.target.value = "";
    },
    []
  );

  const handleSubmit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      if (!items.length || isUploading) return;

      setIsUploading(true);
      setMessage(null);
      setItems((prev) =>
        prev.map((item) => ({
          ...item,
          status: "queued",
          progress: 0,
          error: undefined,
        }))
      );

      const result = await processUploads(items);

      setIsUploading(false);

      if (result.hasError) {
        setMessage("Some uploads failed. Fix issues and retry failed items.");
      } else {
        options?.onSuccess?.();
      }
    },
    [isUploading, items, options, processUploads]
  );

  const retryFailed = useCallback(async () => {
    if (isUploading) return;
    const failedItems = items.filter((item) => item.status === "error");
    if (!failedItems.length) return;

    setIsUploading(true);
    setMessage(null);
    setItems((prev) =>
      prev.map((item) =>
        item.status === "error"
          ? { ...item, status: "queued", progress: 0, error: undefined }
          : item
      )
    );

    const result = await processUploads(failedItems);

    setIsUploading(false);

    if (result.hasError) {
      setMessage("Retry completed with errors. Check failed items.");
    } else {
      options?.onSuccess?.();
    }
  }, [isUploading, items, options, processUploads]);

  const summary = useMemo(() => {
    const total = items.length;
    const success = items.filter((item) => item.status === "success").length;
    const failed = items.filter((item) => item.status === "error").length;
    const inProgress = items.filter((item) => item.status === "encoding" || item.status === "uploading").length;
    return { total, success, failed, inProgress };
  }, [items]);

  return {
    items,
    isUploading,
    message,
    summary,
    handleFileInputChange,
    handleSubmit,
    retryFailed,
  };
}
