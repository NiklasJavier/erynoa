/**
 * FileList Component
 * Display and manage files from S3 storage
 */

import { For, Show, createSignal } from "solid-js";
import type { Component } from "solid-js";
import { useStorageList, useDeleteFile, formatFileSize, getFileIcon } from "../../hooks/useStorage";
import { storage } from "../../api";
import { logger } from "../../lib/logger";

interface FileListProps {
  bucket?: string;
  prefix?: string;
  refreshTrigger?: number;
  onSelect?: (key: string, url: string) => void;
  class?: string;
}

export const FileList: Component<FileListProps> = (props) => {
  const { objects, count, loading, error, refresh } = useStorageList(props.bucket, props.prefix, props.refreshTrigger);
  const { deleteFile, isDeleting } = useDeleteFile();
  const [downloadingKey, setDownloadingKey] = createSignal<string | null>(null);

  const handleDownload = async (key: string) => {
    setDownloadingKey(key);
    try {
      const url = await storage.getDownloadUrl(key, props.bucket);
      // Open in new tab or trigger download
      window.open(url, "_blank");
    } catch (e) {
      logger.error("Download failed", e instanceof Error ? e : new Error(String(e)), {
        component: "FileList",
        action: "download",
        key,
      });
    } finally {
      setDownloadingKey(null);
    }
  };

  const handleDelete = async (key: string) => {
    if (!confirm(`Datei "${key}" wirklich löschen?`)) return;
    
    const success = await deleteFile(key, props.bucket);
    if (success) {
      refresh();
    }
  };

  const handleSelect = async (key: string) => {
    if (!props.onSelect) return;
    
    try {
      const url = await storage.getDownloadUrl(key, props.bucket);
      props.onSelect(key, url);
    } catch (e) {
      logger.error("Failed to get URL", e instanceof Error ? e : new Error(String(e)), {
        component: "FileList",
        action: "getUrl",
        key,
      });
    }
  };

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return "-";
    return new Date(dateStr).toLocaleDateString("de-DE", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <div class={`space-y-4 ${props.class ?? ""}`}>
      {/* Header */}
      <div class="flex items-center justify-between">
        <div class="text-sm text-gray-500 dark:text-gray-400">
          <Show when={!loading()} fallback="Laden...">
            {count()} Datei{count() !== 1 ? "en" : ""}
          </Show>
        </div>
        <button
          onClick={refresh}
          disabled={loading()}
          class="
            px-3 py-1.5 text-sm rounded-lg
            bg-gray-100 dark:bg-gray-700 
            hover:bg-gray-200 dark:hover:bg-gray-600
            disabled:opacity-50 disabled:cursor-not-allowed
            transition-colors
          "
        >
          ↻ Aktualisieren
        </button>
      </div>

      {/* Error State */}
      <Show when={error()}>
        <div class="p-4 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 rounded-lg">
          Fehler: {error()?.message || "Unbekannter Fehler"}
        </div>
      </Show>

      {/* Loading State */}
      <Show when={loading()}>
        <div class="flex items-center justify-center py-8">
          <svg class="animate-spin h-8 w-8 text-blue-500" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
        </div>
      </Show>

      {/* Empty State */}
      <Show when={!loading() && count() === 0}>
        <div class="text-center py-12 text-gray-500 dark:text-gray-400">
          <div class="text-4xl mb-2">📭</div>
          <div>Keine Dateien vorhanden</div>
        </div>
      </Show>

      {/* File List */}
      <Show when={!loading() && count() > 0}>
        <div class="border dark:border-gray-700 rounded-lg overflow-hidden">
          <table class="w-full text-sm">
            <thead class="bg-gray-50 dark:bg-gray-800">
              <tr>
                <th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
                  Datei
                </th>
                <th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300 hidden sm:table-cell">
                  Größe
                </th>
                <th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300 hidden md:table-cell">
                  Geändert
                </th>
                <th class="px-4 py-3 text-right font-medium text-gray-600 dark:text-gray-300">
                  Aktionen
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
              <For each={objects()}>
                {(obj) => (
                  <tr class="hover:bg-gray-50 dark:hover:bg-gray-800/50">
                    <td class="px-4 py-3">
                      <div class="flex items-center gap-2">
                        <span class="text-lg">{getFileIcon(obj.content_type)}</span>
                        <div class="min-w-0">
                          <div
                            class={`
                              truncate text-gray-700 dark:text-gray-200
                              ${props.onSelect ? "cursor-pointer hover:text-blue-500" : ""}
                            `}
                            onClick={() => props.onSelect && handleSelect(obj.key)}
                            title={obj.key}
                          >
                            {obj.key.split("/").pop() || obj.key}
                          </div>
                          <div class="text-xs text-gray-400 truncate hidden lg:block" title={obj.key}>
                            {obj.key}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td class="px-4 py-3 text-gray-500 dark:text-gray-400 hidden sm:table-cell">
                      {formatFileSize(obj.size)}
                    </td>
                    <td class="px-4 py-3 text-gray-500 dark:text-gray-400 hidden md:table-cell">
                      {formatDate(obj.last_modified)}
                    </td>
                    <td class="px-4 py-3">
                      <div class="flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleDownload(obj.key)}
                          disabled={downloadingKey() === obj.key}
                          class="
                            p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700
                            text-gray-500 hover:text-blue-500
                            disabled:opacity-50
                          "
                          title="Herunterladen"
                        >
                          {downloadingKey() === obj.key ? "..." : "⬇️"}
                        </button>
                        <button
                          onClick={() => handleDelete(obj.key)}
                          disabled={isDeleting()}
                          class="
                            p-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700
                            text-gray-500 hover:text-red-500
                            disabled:opacity-50
                          "
                          title="Löschen"
                        >
                          🗑️
                        </button>
                      </div>
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </div>
      </Show>
    </div>
  );
};
