/**
 * Error Boundary Component
 * Catches and displays errors from children
 */

import { createSignal, type ParentComponent, onError } from "solid-js";

interface ErrorInfo {
  message: string;
  stack?: string;
}

export const ErrorBoundary: ParentComponent = (props) => {
  const [error, setError] = createSignal<ErrorInfo | null>(null);

  onError(async (err) => {
    const message = err instanceof Error ? err.message : String(err);
    const stack = err instanceof Error ? err.stack : undefined;
    setError({ message, stack });
    
    // Dynamically import logger to avoid circular dependencies
    try {
      const { logger } = await import("../lib/logger");
      logger.error("Error caught by boundary", err instanceof Error ? err : new Error(message), {
        component: "ErrorBoundary",
        stack,
      });
    } catch (logError) {
      // Fallback to console if logger fails
      console.error("Error caught by boundary:", message, stack);
      console.error("Logger import failed:", logError);
    }
  });

  return (
    <>
      {error() ? (
        <div class="min-h-screen bg-red-50 text-red-900 p-8">
          <div class="max-w-md">
            <h1 class="text-2xl font-bold mb-4">An Error Occurred</h1>
            <pre class="bg-red-100 p-4 rounded text-sm overflow-auto">
              {error()!.message}
              {error()!.stack && `\n\n${error()!.stack}`}
            </pre>
          </div>
        </div>
      ) : (
        props.children
      )}
    </>
  );
};
