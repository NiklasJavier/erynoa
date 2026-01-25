/**
 * FileUpload Component
 * Drag & Drop file upload with progress
 */

import { For, Show } from "solid-js";
import type { Component } from "solid-js";
import { useUpload, useDropzone, formatFileSize } from "../../hooks/useStorage";
import type { UploadState } from "../../hooks/useStorage";

interface FileUploadProps {
  bucket?: string;
  accept?: string;
  multiple?: boolean;
  maxSize?: number; // in bytes
  onUploadComplete?: (results: { key: string; url: string }[]) => void;
  class?: string;
}

export const FileUpload: Component<FileUploadProps> = (props) => {
  const { uploads, isUploading, uploadMultiple, clearUploads, removeUpload } = useUpload(props.bucket);
  let fileInputRef: HTMLInputElement | undefined;

  const handleFiles = async (files: File[]) => {
    // Filter by max size if specified
    const validFiles = props.maxSize
      ? files.filter((f) => f.size <= props.maxSize!)
      : files;

    if (validFiles.length === 0) return;

    const results = await uploadMultiple(validFiles);
    
    if (props.onUploadComplete && results.length > 0) {
      props.onUploadComplete(results.map((r) => ({ key: r.key, url: r.url })));
    }
  };

  const { isDragging, dropzoneProps } = useDropzone(handleFiles);

  const handleFileSelect = (e: Event) => {
    const input = e.target as HTMLInputElement;
    const files = Array.from(input.files || []);
    if (files.length > 0) {
      handleFiles(files);
    }
    // Reset input
    input.value = "";
  };

  const getStatusColor = (status: UploadState["status"]) => {
    switch (status) {
      case "completed":
        return "bg-green-500";
      case "error":
        return "bg-red-500";
      case "uploading":
        return "bg-blue-500";
      default:
        return "bg-gray-400";
    }
  };

  const getStatusIcon = (status: UploadState["status"]) => {
    switch (status) {
      case "completed":
        return "✓";
      case "error":
        return "✕";
      case "uploading":
        return "↑";
      default:
        return "○";
    }
  };

  return (
    <div class={`space-y-4 ${props.class ?? ""}`}>
      {/* Dropzone */}
      <div
        {...dropzoneProps}
        class={`
          relative border-2 border-dashed rounded-lg p-8 text-center cursor-pointer
          transition-colors duration-200 ease-in-out
          ${isDragging()
            ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
            : "border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500"
          }
        `}
        onClick={() => fileInputRef?.click()}
      >
        <input
          ref={fileInputRef}
          type="file"
          accept={props.accept}
          multiple={props.multiple ?? true}
          onChange={handleFileSelect}
          class="hidden"
        />

        <div class="space-y-2">
          <div class="text-4xl">
            {isDragging() ? "📥" : "📤"}
          </div>
          <div class="text-lg font-medium text-gray-700 dark:text-gray-200">
            {isDragging() ? "Dateien hier ablegen" : "Dateien hochladen"}
          </div>
          <div class="text-sm text-gray-500 dark:text-gray-400">
            Drag & Drop oder klicken zum Auswählen
          </div>
          <Show when={props.maxSize}>
            <div class="text-xs text-gray-400 dark:text-gray-500">
              Max. Dateigröße: {formatFileSize(props.maxSize!)}
            </div>
          </Show>
        </div>

        <Show when={isUploading()}>
          <div class="absolute inset-0 bg-white/80 dark:bg-gray-800/80 flex items-center justify-center rounded-lg">
            <div class="flex items-center gap-2">
              <svg class="animate-spin h-5 w-5 text-blue-500" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              <span class="text-gray-700 dark:text-gray-200">Hochladen...</span>
            </div>
          </div>
        </Show>
      </div>

      {/* Upload List */}
      <Show when={uploads().size > 0}>
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-200">
              Uploads ({uploads().size})
            </span>
            <button
              onClick={clearUploads}
              class="text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              Alle entfernen
            </button>
          </div>

          <div class="space-y-2 max-h-64 overflow-y-auto">
            <For each={Array.from(uploads().entries())}>
              {([id, state]) => (
                <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                  {/* Status Icon */}
                  <div
                    class={`
                      w-8 h-8 rounded-full flex items-center justify-center text-white text-sm
                      ${getStatusColor(state.status)}
                    `}
                  >
                    {getStatusIcon(state.status)}
                  </div>

                  {/* File Info */}
                  <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-gray-700 dark:text-gray-200 truncate">
                      {state.file.name}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                      {formatFileSize(state.file.size)}
                      <Show when={state.status === "uploading"}>
                        {" "}• {state.progress}%
                      </Show>
                      <Show when={state.error}>
                        <span class="text-red-500"> • {state.error}</span>
                      </Show>
                    </div>

                    {/* Progress Bar */}
                    <Show when={state.status === "uploading"}>
                      <div class="mt-1 h-1 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                        <div
                          class="h-full bg-blue-500 transition-all duration-300"
                          style={{ width: `${state.progress}%` }}
                        />
                      </div>
                    </Show>
                  </div>

                  {/* Remove Button */}
                  <button
                    onClick={() => removeUpload(id)}
                    class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                  >
                    ✕
                  </button>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
};
