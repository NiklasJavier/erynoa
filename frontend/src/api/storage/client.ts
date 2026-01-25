/**
 * Storage API Client
 * S3-compatible storage operations via backend
 * 
 * REST-based implementation (Connect-RPC available via connect-client.ts)
 */

import { getApiBaseUrl, API_VERSION } from "../../lib/api-config";
import type {
  UploadResult,
  PresignedUrl,
  ListObjectsResponse,
} from "./types";

/**
 * Storage Client für S3-kompatible Operationen
 */
export class StorageClient {
  private baseUrl: string;
  private getToken: () => Promise<string | null>;

  constructor(baseUrl: string, getToken: () => Promise<string | null>) {
    this.baseUrl = baseUrl;
    this.getToken = getToken;
  }

  private async getHeaders(): Promise<HeadersInit> {
    const token = await this.getToken();
    const headers: Record<string, string> = {};
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }
    return headers;
  }

  /**
   * Datei direkt über das Backend hochladen
   */
  async upload(
    file: File,
    bucket?: string,
    onProgress?: (progress: number) => void
  ): Promise<UploadResult> {
    const headers = await this.getHeaders();
    const formData = new FormData();
    formData.append("file", file);

    const endpoint = bucket
      ? `${this.baseUrl}${API_VERSION}/storage/upload/${bucket}`
      : `${this.baseUrl}${API_VERSION}/storage/upload`;

    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();

      xhr.upload.addEventListener("progress", (event) => {
        if (event.lengthComputable && onProgress) {
          const progress = Math.round((event.loaded / event.total) * 100);
          onProgress(progress);
        }
      });

      xhr.addEventListener("load", () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve(JSON.parse(xhr.responseText));
        } else {
          reject(new Error(`Upload failed: ${xhr.statusText}`));
        }
      });

      xhr.addEventListener("error", () => {
        reject(new Error("Upload failed: Network error"));
      });

      xhr.open("POST", endpoint);
      
      // Set auth header
      const authHeader = (headers as Record<string, string>)["Authorization"];
      if (authHeader) {
        xhr.setRequestHeader("Authorization", authHeader);
      }

      xhr.send(formData);
    });
  }

  /**
   * Presigned URL für direkten Upload generieren
   * Ermöglicht Upload direkt zu S3 ohne Backend-Umweg
   */
  async getPresignedUploadUrl(
    key: string,
    bucket?: string
  ): Promise<PresignedUrl> {
    const headers = await this.getHeaders();
    const params = new URLSearchParams();
    if (bucket) params.append("bucket", bucket);

    const response = await fetch(
      `${this.baseUrl}${API_VERSION}/storage/presigned/upload/${key}?${params}`,
      { headers }
    );

    if (!response.ok) {
      throw new Error(`Failed to get presigned upload URL: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Presigned URL für Download generieren
   */
  async getPresignedDownloadUrl(
    key: string,
    bucket?: string
  ): Promise<PresignedUrl> {
    const headers = await this.getHeaders();
    const params = new URLSearchParams();
    if (bucket) params.append("bucket", bucket);

    const response = await fetch(
      `${this.baseUrl}${API_VERSION}/storage/presigned/download/${key}?${params}`,
      { headers }
    );

    if (!response.ok) {
      throw new Error(`Failed to get presigned download URL: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Datei direkt über Presigned URL hochladen (für große Dateien)
   */
  async uploadDirect(
    file: File,
    presignedUrl: string,
    onProgress?: (progress: number) => void
  ): Promise<void> {
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();

      xhr.upload.addEventListener("progress", (event) => {
        if (event.lengthComputable && onProgress) {
          const progress = Math.round((event.loaded / event.total) * 100);
          onProgress(progress);
        }
      });

      xhr.addEventListener("load", () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve();
        } else {
          reject(new Error(`Upload failed: ${xhr.statusText}`));
        }
      });

      xhr.addEventListener("error", () => {
        reject(new Error("Upload failed: Network error"));
      });

      xhr.open("PUT", presignedUrl);
      xhr.setRequestHeader("Content-Type", file.type || "application/octet-stream");
      xhr.send(file);
    });
  }

  /**
   * Objekte im Bucket auflisten
   */
  async list(prefix?: string, bucket?: string): Promise<ListObjectsResponse> {
    const headers = await this.getHeaders();
    const params = new URLSearchParams();
    if (prefix) params.append("prefix", prefix);
    if (bucket) params.append("bucket", bucket);

    const response = await fetch(
      `${this.baseUrl}${API_VERSION}/storage/list?${params}`,
      { headers }
    );

    if (!response.ok) {
      throw new Error(`Failed to list objects: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Objekt löschen
   */
  async delete(key: string, bucket?: string): Promise<void> {
    const headers = await this.getHeaders();
    const params = new URLSearchParams();
    if (bucket) params.append("bucket", bucket);

    const response = await fetch(
      `${this.baseUrl}${API_VERSION}/storage/${key}?${params}`,
      { method: "DELETE", headers }
    );

    if (!response.ok) {
      throw new Error(`Failed to delete object: ${response.statusText}`);
    }
  }

  /**
   * Prüfen ob Objekt existiert
   */
  async exists(key: string, bucket?: string): Promise<boolean> {
    const headers = await this.getHeaders();
    const params = new URLSearchParams();
    if (bucket) params.append("bucket", bucket);

    const response = await fetch(
      `${this.baseUrl}${API_VERSION}/storage/${key}?${params}`,
      { method: "HEAD", headers }
    );

    return response.ok;
  }

  /**
   * Download-URL für eine Datei generieren
   */
  async getDownloadUrl(key: string, bucket?: string): Promise<string> {
    const { url } = await this.getPresignedDownloadUrl(key, bucket);
    return url;
  }
}

// Singleton instance
let storageClient: StorageClient | null = null;

export function initStorageClient(getToken: () => Promise<string | null>) {
  storageClient = new StorageClient(getApiBaseUrl(), getToken);
  return storageClient;
}

export function getStorageClient(): StorageClient {
  if (!storageClient) {
    // Return a client without auth
    return new StorageClient(getApiBaseUrl(), async () => null);
  }
  return storageClient;
}

// Convenience exports
export const storage = {
  upload: (file: File, bucket?: string, onProgress?: (p: number) => void) =>
    getStorageClient().upload(file, bucket, onProgress),
  list: (prefix?: string, bucket?: string) =>
    getStorageClient().list(prefix, bucket),
  delete: (key: string, bucket?: string) =>
    getStorageClient().delete(key, bucket),
  exists: (key: string, bucket?: string) =>
    getStorageClient().exists(key, bucket),
  getDownloadUrl: (key: string, bucket?: string) =>
    getStorageClient().getDownloadUrl(key, bucket),
  getPresignedUploadUrl: (key: string, bucket?: string) =>
    getStorageClient().getPresignedUploadUrl(key, bucket),
};
