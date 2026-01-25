/**
 * Storage Service API
 * 
 * Feature-based organization for Storage Service
 * Uses Connect-RPC for all storage operations
 */

// Re-export types (Protobuf as primary source)
export * from "./types";

// Export both clients for flexibility
export { StorageClient } from "./client";
export { ConnectStorageClient } from "./connect-client";

// Use Connect-RPC client as default
import { ConnectStorageClient } from "./connect-client";

let storageClient: ConnectStorageClient | null = null;

export function initStorageClient(getToken: () => Promise<string | null>) {
  storageClient = new ConnectStorageClient(getToken);
  return storageClient;
}

export function getStorageClient(): ConnectStorageClient {
  if (!storageClient) {
    // Return a client without auth for initial operations
    return new ConnectStorageClient(async () => null);
  }
  return storageClient;
}

// Convenience exports using Connect-RPC
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
  getPresignedUploadUrl: (key: string, bucket?: string, expiresIn?: number) =>
    getStorageClient().getPresignedUploadUrl(key, bucket, expiresIn),
  listBuckets: () => getStorageClient().listBuckets(),
  createBucket: (name: string) => getStorageClient().createBucket(name),
  deleteBucket: (name: string) => getStorageClient().deleteBucket(name),
};

// Re-export types (from feature module)
export type {
  StorageObject,
  UploadResult,
  PresignedUrl,
  ListObjectsResponse,
} from "./types";
