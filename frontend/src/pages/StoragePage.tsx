/**
 * Storage Page
 * Complete bucket and file management with tree navigation
 */

import type { Component } from "solid-js";
import {
  createSignal,
  createEffect,
  createMemo,
  Show,
  For,
  batch,
} from "solid-js";
import StorageBrowser from "../components/storage/StorageBrowser";
import { useUpload } from "../hooks/useStorage";
import { storage, type StorageObject } from "../api";

interface BucketInfo {
  name: string;
  size: number;
  files: number;
  created?: string;
}

const StoragePage: Component = () => {
  const [buckets, setBuckets] = createSignal<string[]>(["uploads"]);
  const [selectedBucket, setSelectedBucket] = createSignal("uploads");
  const [refreshTrigger, setRefreshTrigger] = createSignal(0);
  const [bucketInfo, setBucketInfo] = createSignal<Record<string, BucketInfo>>({});
  const [showUploadModal, setShowUploadModal] = createSignal(false);
  const [showNewBucketModal, setShowNewBucketModal] = createSignal(false);
  const [newBucketName, setNewBucketName] = createSignal("");
  const [loadingBuckets, setLoadingBuckets] = createSignal(false);
  
  // Create upload hook as memo that updates when selectedBucket changes
  const uploadHook = createMemo(() => useUpload(selectedBucket()));
  const upload = createMemo(() => uploadHook().upload);
  const isUploading = createMemo(() => uploadHook().isUploading);

  // Load all buckets on mount
  createEffect(() => {
    loadBuckets();
  });

  const loadBuckets = async () => {
    setLoadingBuckets(true);
    try {
      const response = await fetch("/api/v1/storage/buckets", {
        method: "GET",
      });
      if (response.ok) {
        const data = await response.json();
        setBuckets(data.buckets || ["uploads"]);
        if (!buckets().includes(selectedBucket())) {
          setSelectedBucket(buckets()[0]);
        }
      }
    } catch (error) {
      console.error("Failed to load buckets:", error);
    } finally {
      setLoadingBuckets(false);
    }
  };

  const createBucket = async () => {
    const name = newBucketName().trim();
    if (!name) {
      alert("Bucket-Name ist erforderlich");
      return;
    }

    try {
      const response = await fetch("/api/v1/storage/buckets", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      });

      if (response.ok) {
        setNewBucketName("");
        setShowNewBucketModal(false);
        await loadBuckets();
        setSelectedBucket(name);
      } else {
        alert("Bucket konnte nicht erstellt werden");
      }
    } catch (error) {
      console.error("Failed to create bucket:", error);
      alert("Fehler beim Erstellen des Buckets");
    }
  };

  const deleteBucket = async (bucket: string) => {
    if (
      !confirm(
        `Möchtest du den Bucket "${bucket}" wirklich löschen? Dies kann nicht rückgängig gemacht werden.`
      )
    ) {
      return;
    }

    try {
      const response = await fetch(`/api/v1/storage/buckets/${bucket}`, {
        method: "DELETE",
      });

      if (response.ok) {
        await loadBuckets();
        if (selectedBucket() === bucket) {
          setSelectedBucket(buckets()[0]);
        }
      } else {
        alert("Bucket konnte nicht gelöscht werden");
      }
    } catch (error) {
      console.error("Failed to delete bucket:", error);
      alert("Fehler beim Löschen des Buckets");
    }
  };

  const handleUpload = async (event: Event) => {
    const input = event.target as HTMLInputElement;
    const files = Array.from(input.files || []);

    for (const file of files) {
      try {
        await upload()(file);
        setRefreshTrigger((k) => k + 1);
      } catch (error) {
        console.error("Upload failed:", error);
      }
    }

    // Reset input
    input.value = "";
  };

  const handleDragOver = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();
  };

  const handleDrop = async (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();
    const files = Array.from(event.dataTransfer?.files || []);

    for (const file of files) {
      try {
        await upload()(file);
        setRefreshTrigger((k) => k + 1);
      } catch (error) {
        console.error("Upload failed:", error);
      }
    }
  };

  const selectBucket = (bucket: string) => {
    batch(() => {
      setSelectedBucket(bucket);
      setRefreshTrigger((k) => k + 1);
    });
  };

  return (
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div class="max-w-7xl mx-auto px-4 py-8">
        {/* Header */}
        <div class="mb-8">
          <h1 class="text-4xl font-bold text-gray-800 dark:text-white mb-2">
            📦 Storage Management
          </h1>
          <p class="text-gray-600 dark:text-gray-400">
            Verwalte deine Buckets und Dateien mit Baum-Navigation
          </p>
        </div>

        {/* Top Toolbar */}
        <div class="mb-6 flex gap-3 flex-wrap">
          <button
            onClick={() => setShowUploadModal(true)}
            disabled={isUploading()}
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 flex items-center gap-2"
          >
            📤 {isUploading() ? "Wird hochgeladen..." : "Hochladen"}
          </button>
          <button
            onClick={() => setShowNewBucketModal(true)}
            class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center gap-2"
          >
            ➕ Neuer Bucket
          </button>
          <button
            onClick={() => loadBuckets()}
            disabled={loadingBuckets()}
            class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors disabled:opacity-50"
          >
            🔄 Aktualisieren
          </button>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
          {/* Bucket Sidebar */}
          <div class="lg:col-span-1">
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
              <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <h3 class="font-semibold text-gray-900 dark:text-white">
                  Buckets
                </h3>
              </div>
              <div class="divide-y divide-gray-200 dark:divide-gray-700">
                <Show
                  when={buckets().length > 0}
                  fallback={
                    <div class="p-4 text-gray-500 text-sm">Keine Buckets</div>
                  }
                >
                  <For each={buckets()}>
                    {(bucket) => (
                      <div class="p-0">
                        <button
                          onClick={() => selectBucket(bucket)}
                          class={`
                            w-full text-left px-4 py-3 transition-colors
                            ${
                              selectedBucket() === bucket
                                ? "bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100 border-l-4 border-blue-600"
                                : "hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-900 dark:text-white"
                            }
                          `}
                        >
                          <div class="flex items-center justify-between">
                            <span class="font-medium">{bucket}</span>
                            <Show when={selectedBucket() === bucket}>
                              <span class="text-xs">✓</span>
                            </Show>
                          </div>
                        </button>
                        <Show when={selectedBucket() === bucket}>
                          <div class="px-4 py-2 bg-gray-50 dark:bg-gray-700 flex gap-2">
                            <button
                              onClick={() => deleteBucket(bucket)}
                              disabled={buckets().length === 1}
                              class="flex-1 text-xs px-2 py-1 bg-red-100 text-red-700 rounded hover:bg-red-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                              🗑️
                            </button>
                          </div>
                        </Show>
                      </div>
                    )}
                  </For>
                </Show>
              </div>
            </div>
          </div>

          {/* File Browser */}
          <div class="lg:col-span-3">
            <div
              class="bg-white dark:bg-gray-800 rounded-lg shadow h-screen overflow-hidden flex flex-col"
              onDragOver={handleDragOver}
              onDrop={handleDrop}
            >
              {/* Upload Zone */}
              <div class="border-b border-gray-200 dark:border-gray-700 p-4 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20">
                <label class="block cursor-pointer">
                  <input
                    type="file"
                    multiple
                    onChange={handleUpload}
                    disabled={isUploading()}
                    class="hidden"
                  />
                  <div class="border-2 border-dashed border-blue-300 dark:border-blue-600 rounded-lg p-6 text-center hover:border-blue-500 dark:hover:border-blue-400 transition-colors">
                    <p class="text-blue-700 dark:text-blue-300 font-medium">
                      📁 Dateien hier ablegen oder klicken zum Upload
                    </p>
                    <p class="text-sm text-blue-600 dark:text-blue-400 mt-1">
                      {isUploading()
                        ? "Wird hochgeladen..."
                        : "Zu: " + selectedBucket()}
                    </p>
                  </div>
                </label>
              </div>

              {/* Browser */}
              <div class="flex-1 overflow-hidden">
                <StorageBrowser
                  bucket={selectedBucket()}
                  refreshTrigger={refreshTrigger()}
                  class="h-full"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Upload Modal */}
      <Show when={showUploadModal()}>
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div class="bg-white dark:bg-gray-800 rounded-lg p-8 max-w-md w-full mx-4">
            <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-4">
              Datei hochladen
            </h3>
            <label class="block">
              <input
                type="file"
                multiple
                onChange={(e) => {
                  handleUpload(e);
                  setShowUploadModal(false);
                }}
                disabled={isUploading()}
                class="w-full"
              />
            </label>
            <div class="mt-4 flex gap-2 justify-end">
              <button
                onClick={() => setShowUploadModal(false)}
                disabled={isUploading()}
                class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
              >
                Abbrechen
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* New Bucket Modal */}
      <Show when={showNewBucketModal()}>
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div class="bg-white dark:bg-gray-800 rounded-lg p-8 max-w-md w-full mx-4">
            <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-4">
              Neuer Bucket
            </h3>
            <input
              type="text"
              placeholder="Bucket-Name"
              value={newBucketName()}
              onInput={(e) => setNewBucketName(e.currentTarget.value)}
              class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 mb-4"
            />
            <div class="flex gap-2 justify-end">
              <button
                onClick={() => {
                  setShowNewBucketModal(false);
                  setNewBucketName("");
                }}
                class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
              >
                Abbrechen
              </button>
              <button
                onClick={createBucket}
                class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
              >
                Erstellen
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default StoragePage;
