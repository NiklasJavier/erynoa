/**
 * StorageBrowser Component
 * Tree-based file browser for S3 storage with search, selection, and actions
 */

import type { Component as SolidComponent } from "solid-js";
import {
  createSignal,
  createEffect,
  For,
  Show,
  batch,
  createMemo,
} from "solid-js";
import { storage, type StorageObject } from "../../api";
import { useStorageList, useDeleteFile } from "../../hooks/useStorage";
import { logger } from "../../lib/logger";

interface FileNode extends StorageObject {
  path: string;
  isFolder: boolean;
  children?: FileNode[];
  expanded?: boolean;
  level: number;
}

interface StorageBrowserProps {
  bucket?: string;
  refreshTrigger?: number;
  onSelect?: (key: string, url: string) => void;
  class?: string;
}

const getFileIcon = (key: string, isFolder: boolean): string => {
  if (isFolder) return "📁";
  const ext = key.split(".").pop()?.toLowerCase() || "";
  const iconMap: Record<string, string> = {
    pdf: "📄",
    doc: "📝",
    docx: "📝",
    txt: "📄",
    jpg: "🖼️",
    jpeg: "🖼️",
    png: "🖼️",
    gif: "🎬",
    mp4: "🎥",
    mp3: "🎵",
    zip: "📦",
    rar: "📦",
    "7z": "📦",
    csv: "📊",
    xlsx: "📊",
    xls: "📊",
  };
  return iconMap[ext] || "📄";
};

const buildTreeFromObjects = (objects: StorageObject[]): FileNode[] => {
  const root: Record<string, FileNode> = {};

  // Create all nodes
  objects.forEach((obj) => {
    const parts = obj.key.split("/").filter(Boolean);
    let current = root;
    let path = "";

    parts.forEach((part, index) => {
      path = path ? `${path}/${part}` : part;
      const isLast = index === parts.length - 1;

      if (!current[part]) {
        current[part] = {
          ...obj,
          key: isLast ? obj.key : path,
          path,
          size: obj.size || 0,
          isFolder: !isLast || obj.key.endsWith("/"),
          level: index,
          children: [],
          expanded: false,
        };
      }

      if (current[part].children) {
        current = current[part].children as unknown as Record<string, FileNode>;
      }
    });
  });

  const sortNodes = (nodes: FileNode[]): FileNode[] => {
    return nodes.sort((a, b) => {
      if (a.isFolder !== b.isFolder) return a.isFolder ? -1 : 1;
      return a.path.localeCompare(b.path);
    });
  };

  const flattenTree = (nodes: Record<string, FileNode>): FileNode[] => {
    const result: FileNode[] = [];
    Object.values(nodes).forEach((node) => {
      if (node.children?.length) {
        node.children = sortNodes(node.children);
      }
      result.push(node);
    });
    return sortNodes(result);
  };

  return flattenTree(root);
};

export const StorageBrowser: SolidComponent<StorageBrowserProps> = (props) => {
  const { objects, loading, error, refresh } = useStorageList(
    props.bucket,
    "",
    props.refreshTrigger
  );
  const { deleteFile, isDeleting } = useDeleteFile();

  const [searchTerm, setSearchTerm] = createSignal<string>("");
  const [expandedPaths, setExpandedPaths] = createSignal<Set<string>>(
    new Set()
  );
  const [selectedPaths, setSelectedPaths] = createSignal<Set<string>>(
    new Set()
  );
  const [lastClickedPath, setLastClickedPath] = createSignal<string | null>(
    null
  );
  const [downloadingKey, setDownloadingKey] = createSignal<string | null>(
    null
  );
  const [contextMenu, setContextMenu] = createSignal<{
    path: string;
    x: number;
    y: number;
  } | null>(null);

  // Build tree structure from flat object list
  const treeNodes = createMemo(() => {
    return buildTreeFromObjects(objects() || []);
  });

  // Filter nodes by search term
  const filteredNodes = createMemo(() => {
    const search = searchTerm().toLowerCase();
    if (!search) return treeNodes();

    const filter = (nodes: FileNode[]): FileNode[] => {
      return nodes
        .map((node) => ({
          ...node,
          children: node.children ? filter(node.children) : [],
        }))
        .filter(
          (node) =>
            node.path.toLowerCase().includes(search) ||
            (node.children && node.children.length > 0)
        );
    };

    return filter(treeNodes());
  });

  // Flatten tree into virtual list
  const flatList = createMemo(() => {
    const result: FileNode[] = [];
    const expanded = expandedPaths();

    const traverse = (nodes: FileNode[]) => {
      nodes.forEach((node) => {
        result.push(node);
        if (node.isFolder && expanded.has(node.path) && node.children) {
          traverse(node.children);
        }
      });
    };

    traverse(filteredNodes());
    return result;
  });

  const toggleFolder = (path: string) => {
    batch(() => {
      setExpandedPaths((prev) => {
        const next = new Set<string>(prev);
        if (next.has(path)) {
          next.delete(path);
        } else {
          next.add(path);
        }
        return next;
      });
    });
  };

  const toggleSelect = (
    path: string,
    event: MouseEvent & { currentTarget: HTMLElement }
  ) => {
    batch(() => {
      if (event.ctrlKey || event.metaKey) {
        // Ctrl/Cmd: toggle single selection
        setSelectedPaths((prev) => {
          const next = new Set<string>(prev);
          if (next.has(path)) {
            next.delete(path);
          } else {
            next.add(path);
          }
          return next;
        });
        setLastClickedPath(path);
      } else if (event.shiftKey) {
        // Shift: range selection
        const last = lastClickedPath();
        if (last) {
          const allPaths = flatList().map((n) => n.path);
          const lastIdx = allPaths.indexOf(last);
          const currentIdx = allPaths.indexOf(path);
          if (lastIdx !== -1 && currentIdx !== -1) {
            const [start, end] = [lastIdx, currentIdx].sort(
              (a, b) => a - b
            );
            setSelectedPaths(new Set<string>(allPaths.slice(start, end + 1)));
          }
        }
      } else {
        // Single selection
        setSelectedPaths(new Set<string>([path]));
        setLastClickedPath(path);
      }
    });
  };

  const handleContextMenu = (
    path: string,
    event: MouseEvent & { currentTarget: HTMLElement }
  ) => {
    event.preventDefault();
    // Ensure clicked item is selected
    if (!selectedPaths().has(path)) {
      setSelectedPaths(new Set<string>([path]));
    }
    setContextMenu({
      path,
      x: event.clientX,
      y: event.clientY,
    });
  };

  const handleDownload = async (key: string) => {
    setDownloadingKey(key);
    try {
      const url = await storage.getDownloadUrl(key, props.bucket);
      window.open(url, "_blank");
    } catch (e) {
      logger.error("Download failed", e instanceof Error ? e : new Error(String(e)), {
        component: "StorageBrowser",
        action: "download",
        key,
      });
    } finally {
      setDownloadingKey(null);
    }
  };

  const handleDelete = async (key: string) => {
    if (!confirm(`Datei/Ordner "${key}" wirklich löschen?`)) return;
    const success = await deleteFile(key, props.bucket);
    if (success) {
      refresh();
      setSelectedPaths(new Set<string>());
    }
  };

  const handleDeleteMultiple = async () => {
    const selected = Array.from(selectedPaths());
    if (!confirm(`${selected.length} Datei(en) wirklich löschen?`)) return;

    for (const path of selected) {
      await deleteFile(path, props.bucket);
    }
    refresh();
    setSelectedPaths(new Set<string>());
  };

  const handleOpen = async (node: FileNode) => {
    if (node.isFolder) {
      toggleFolder(node.path);
    } else if (props.onSelect) {
      try {
        const url = await storage.getDownloadUrl(node.key, props.bucket);
        props.onSelect(node.key, url);
      } catch (e) {
        logger.error("Failed to open file", e instanceof Error ? e : new Error(String(e)), {
          component: "StorageBrowser",
          action: "openFile",
          key: node.key,
        });
      }
    }
  };

  const handleDoubleClick = (node: FileNode) => {
    if (!node.isFolder) {
      handleOpen(node);
    }
  };

  // Close context menu on click
  createEffect(() => {
    if (contextMenu()) {
      const handler = () => setContextMenu(null);
      document.addEventListener("click", handler);
      return () => document.removeEventListener("click", handler);
    }
  });

  return (
    <div class={`flex flex-col h-full bg-white dark:bg-gray-900 ${props.class || ""}`}>
      {/* Header with Search */}
      <div class="p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="relative mb-3">
          <input
            type="text"
            placeholder="🔍 Dateien durchsuchen..."
            value={searchTerm()}
            onInput={(e) => setSearchTerm(e.currentTarget.value)}
            class="w-full px-4 py-2 pl-10 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Action Buttons */}
        <div class="flex gap-2 flex-wrap">
          <Show when={selectedPaths().size > 0}>
            <button
              onClick={handleDeleteMultiple}
              class="px-3 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600 transition-colors"
            >
              🗑️ {selectedPaths().size} löschen
            </button>
            <button
              onClick={() => {
                const first = flatList().find((n) =>
                  selectedPaths().has(n.path)
                );
                if (first && !first.isFolder) {
                  handleDownload(first.key);
                }
              }}
              class="px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors disabled:opacity-50"
              disabled={downloadingKey() !== null}
            >
              ⬇️ Herunterladen
            </button>
          </Show>
          <button
            onClick={() => refresh()}
            disabled={loading()}
            class="px-3 py-1 text-sm bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-white rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors disabled:opacity-50"
          >
            🔄 Aktualisieren
          </button>
        </div>

        {/* Status */}
        <Show when={error()}>
          <div class="mt-2 text-red-600 dark:text-red-400 text-sm">
            Fehler: {error()}
          </div>
        </Show>
      </div>

      {/* File Tree */}
      <div class="flex-1 overflow-auto">
        <Show
          when={!loading()}
          fallback={<div class="p-4 text-gray-500">Dateien werden geladen...</div>}
        >
          <div class="divide-y divide-gray-100 dark:divide-gray-800">
            <For each={flatList()}>
              {(node) => (
                <div
                  onClick={(e) => toggleSelect(node.path, e)}
                  onContextMenu={(e) => handleContextMenu(node.path, e)}
                  onDblClick={() => handleDoubleClick(node)}
                  class={`
                    flex items-center px-2 py-2 cursor-pointer transition-colors
                    ${selectedPaths().has(node.path)
                      ? "bg-blue-100 dark:bg-blue-900"
                      : "hover:bg-gray-50 dark:hover:bg-gray-800"
                    }
                  `}
                  style={{
                    "padding-left": `${node.level * 16 + 8}px`,
                  }}
                >
                  {/* Expander */}
                  <Show when={node.isFolder}>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleFolder(node.path);
                      }}
                      class="mr-1 w-5 text-center"
                    >
                      {expandedPaths().has(node.path) ? "▼" : "▶"}
                    </button>
                  </Show>

                  {/* Spacer for non-folder items */}
                  <Show when={!node.isFolder}>
                    <div class="mr-1 w-5"></div>
                  </Show>

                  {/* Icon and Name */}
                  <span class="mr-2 text-lg">
                    {getFileIcon(node.path, node.isFolder)}
                  </span>
                  <span class="flex-1 truncate text-sm text-gray-900 dark:text-gray-100">
                    {node.path.split("/").pop() || node.path}
                  </span>

                  {/* Size */}
                  <Show when={!node.isFolder && node.size}>
                    <span class="ml-2 text-xs text-gray-500 dark:text-gray-400">
                      {(node.size / 1024).toFixed(1)} KB
                    </span>
                  </Show>

                  {/* Loading indicator */}
                  <Show when={downloadingKey() === node.key}>
                    <span class="ml-2 text-xs text-blue-500">⏳</span>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </Show>

        <Show when={flatList().length === 0 && !loading()}>
          <div class="p-4 text-center text-gray-500 dark:text-gray-400">
            {searchTerm() ? "Keine Dateien gefunden" : "Keine Dateien"}
          </div>
        </Show>
      </div>

      {/* Context Menu */}
      <Show when={contextMenu()}>
        {(menu) => (
          <div
            class="fixed bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-50"
            style={{
              left: `${menu().x}px`,
              top: `${menu().y}px`,
            }}
          >
            {(() => {
              const node = flatList().find((n) => n.path === menu().path);
              if (!node) return null;

              return (
                <div class="py-1">
                  <Show when={!node.isFolder}>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDownload(node.key);
                        setContextMenu(null);
                      }}
                      class="block w-full text-left px-4 py-2 text-sm text-gray-900 dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700"
                    >
                      ⬇️ Herunterladen
                    </button>
                    <button
                      onClick={() => {
                        handleOpen(node);
                        setContextMenu(null);
                      }}
                      class="block w-full text-left px-4 py-2 text-sm text-gray-900 dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700"
                    >
                      🔗 Öffnen
                    </button>
                  </Show>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(node.key);
                      setContextMenu(null);
                    }}
                    disabled={isDeleting()}
                    class="block w-full text-left px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-gray-100 dark:hover:bg-gray-700 disabled:opacity-50"
                  >
                    🗑️ Löschen
                  </button>
                </div>
              );
            })()}
          </div>
        )}
      </Show>
    </div>
  );
};
