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

  onError((err) => {
    const message = err instanceof Error ? err.message : String(err);
    const stack = err instanceof Error ? err.stack : undefined;
    setError({ message, stack });
    console.error("Error caught by boundary:", message, stack);
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
