/**
 * Storage Hooks
 * SolidJS hooks for S3 storage operations
 */

import { createSignal, createResource } from "solid-js";
import { storage, type UploadResult } from "../api";

export interface UploadState {
  file: File;
  progress: number;
  status: "pending" | "uploading" | "completed" | "error";
  result?: UploadResult;
  error?: string;
}

/**
 * Hook für Datei-Upload mit Progress-Tracking
 */
export function useUpload(bucket?: string) {
  const [uploads, setUploads] = createSignal<Map<string, UploadState>>(new Map());
  const [isUploading, setIsUploading] = createSignal(false);

  const upload = async (file: File): Promise<UploadResult | null> => {
    const id = `${file.name}-${Date.now()}`;
    
    // Add to uploads map
    setUploads((prev) => {
      const next = new Map(prev);
      next.set(id, {
        file,
        progress: 0,
        status: "pending",
      });
      return next;
    });

    setIsUploading(true);

    try {
      // Update status to uploading
      setUploads((prev) => {
        const next = new Map(prev);
        const state = next.get(id);
        if (state) {
          next.set(id, { ...state, status: "uploading" });
        }
        return next;
      });

      const result = await storage.upload(file, bucket, (progress) => {
        setUploads((prev) => {
          const next = new Map(prev);
          const state = next.get(id);
          if (state) {
            next.set(id, { ...state, progress });
          }
          return next;
        });
      });

      // Update status to completed
      setUploads((prev) => {
        const next = new Map(prev);
        const state = next.get(id);
        if (state) {
          next.set(id, { ...state, status: "completed", progress: 100, result });
        }
        return next;
      });

      return result;
    } catch (error) {
      // Update status to error
      setUploads((prev) => {
        const next = new Map(prev);
        const state = next.get(id);
        if (state) {
          next.set(id, {
            ...state,
            status: "error",
            error: error instanceof Error ? error.message : "Upload failed",
          });
        }
        return next;
      });
      return null;
    } finally {
      setIsUploading(false);
    }
  };

  const uploadMultiple = async (files: File[]): Promise<UploadResult[]> => {
    const results: UploadResult[] = [];
    for (const file of files) {
      const result = await upload(file);
      if (result) results.push(result);
    }
    return results;
  };

  const clearUploads = () => {
    setUploads(new Map());
  };

  const removeUpload = (id: string) => {
    setUploads((prev) => {
      const next = new Map(prev);
      next.delete(id);
      return next;
    });
  };

  return {
    uploads,
    isUploading,
    upload,
    uploadMultiple,
    clearUploads,
    removeUpload,
  };
}

/**
 * Hook für Objekt-Liste
 */
export function useStorageList(bucket?: string, prefix?: string, refreshTrigger?: number) {
  const [refetchTrigger, setRefetchTrigger] = createSignal(0);

  const [data, { refetch }] = createResource(
    () => ({ bucket, prefix, trigger: refetchTrigger(), externalTrigger: refreshTrigger }),
    async ({ bucket, prefix }) => {
      return storage.list(prefix, bucket);
    }
  );

  const refresh = () => {
    setRefetchTrigger((n) => n + 1);
  };

  return {
    objects: () => data()?.objects ?? [],
    count: () => data()?.count ?? 0,
    loading: () => data.loading,
    error: () => data.error,
    refresh,
    refetch,
  };
}

/**
 * Hook für Datei-Löschung
 */
export function useDeleteFile() {
  const [isDeleting, setIsDeleting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const deleteFile = async (key: string, bucket?: string): Promise<boolean> => {
    setIsDeleting(true);
    setError(null);

    try {
      await storage.delete(key, bucket);
      return true;
    } catch (e) {
      setError(e instanceof Error ? e.message : "Delete failed");
      return false;
    } finally {
      setIsDeleting(false);
    }
  };

  return {
    deleteFile,
    isDeleting,
    error,
  };
}

/**
 * Hook für Download-URL
 */
export function useDownloadUrl(key: string, bucket?: string) {
  const [url, setUrl] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const fetchUrl = async () => {
    setLoading(true);
    setError(null);

    try {
      const downloadUrl = await storage.getDownloadUrl(key, bucket);
      setUrl(downloadUrl);
      return downloadUrl;
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to get download URL");
      return null;
    } finally {
      setLoading(false);
    }
  };

  return {
    url,
    loading,
    error,
    fetchUrl,
  };
}

/**
 * Drag & Drop Hook für File-Uploads
 */
export function useDropzone(onDrop: (files: File[]) => void) {
  const [isDragging, setIsDragging] = createSignal(false);

  const handleDragEnter = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length > 0) {
      onDrop(files);
    }
  };

  return {
    isDragging,
    dropzoneProps: {
      onDragEnter: handleDragEnter,
      onDragLeave: handleDragLeave,
      onDragOver: handleDragOver,
      onDrop: handleDrop,
    },
  };
}

/**
 * Format file size to human readable
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Get file icon based on content type
 */
export function getFileIcon(contentType?: string): string {
  if (!contentType) return "📄";
  
  if (contentType.startsWith("image/")) return "🖼️";
  if (contentType.startsWith("video/")) return "🎥";
  if (contentType.startsWith("audio/")) return "🎵";
  if (contentType.startsWith("text/")) return "📝";
  if (contentType.includes("pdf")) return "📕";
  if (contentType.includes("zip") || contentType.includes("compressed")) return "📦";
  if (contentType.includes("spreadsheet") || contentType.includes("excel")) return "📊";
  if (contentType.includes("document") || contentType.includes("word")) return "📄";
  
  return "📄";
}
