/**
 * Error Boundary Component
 * Catches and displays errors from children
 * 
 * Features:
 * - Handles Connect-RPC errors with user-friendly messages
 * - Maps error codes to readable messages
 * - Shows service/method context for RPC errors
 */

import { createSignal, type ParentComponent, onError } from "solid-js";

interface ErrorInfo {
  message: string;
  stack?: string;
  code?: string;
  service?: string;
  method?: string;
  userMessage?: string;
}

/**
 * Check if error is a Connect-RPC error
 */
async function isConnectError(error: unknown): Promise<boolean> {
  try {
    const { ConnectError } = await import("@connectrpc/connect");
    return error instanceof ConnectError;
  } catch {
    return false;
  }
}

/**
 * Extract user-friendly message from Connect-RPC error
 */
async function getConnectErrorMessage(error: unknown): Promise<{
  userMessage: string;
  code?: string;
  service?: string;
  method?: string;
}> {
  try {
    const { ConnectError } = await import("@connectrpc/connect");
    
    if (error instanceof ConnectError) {
      // Map error codes to user-friendly messages
      const codeMessages: Record<string, string> = {
        "unauthenticated": "Du bist nicht angemeldet. Bitte melde dich an.",
        "permission_denied": "Du hast keine Berechtigung für diese Aktion.",
        "not_found": "Die angeforderte Ressource wurde nicht gefunden.",
        "invalid_argument": "Die Anfrage ist ungültig. Bitte überprüfe deine Eingaben.",
        "internal": "Ein interner Fehler ist aufgetreten. Bitte versuche es später erneut.",
        "unavailable": "Der Service ist derzeit nicht verfügbar. Bitte versuche es später erneut.",
        "deadline_exceeded": "Die Anfrage hat zu lange gedauert. Bitte versuche es erneut.",
        "resource_exhausted": "Zu viele Anfragen. Bitte warte einen Moment.",
        "failed_precondition": "Die Anfrage kann nicht ausgeführt werden. Bitte überprüfe die Voraussetzungen.",
        "aborted": "Die Anfrage wurde abgebrochen.",
        "out_of_range": "Die Anfrage liegt außerhalb des gültigen Bereichs.",
        "unimplemented": "Diese Funktion ist noch nicht implementiert.",
        "cancelled": "Die Anfrage wurde abgebrochen.",
        "already_exists": "Die Ressource existiert bereits.",
        "data_loss": "Datenverlust aufgetreten. Bitte kontaktiere den Support.",
      };

      // Convert error.code to string for comparison
      const codeStr = String(error.code);
      const userMessage = codeMessages[codeStr] || 
        `Ein Fehler ist aufgetreten: ${error.message}`;

      // Try to extract service and method from error message
      // Format: "ServiceName.methodName: [CODE] message"
      const match = error.message.match(/^([^.]+)\.([^:]+):/);
      const service = match?.[1];
      const method = match?.[2];

      return {
        userMessage,
        code: codeStr,
        service,
        method,
      };
    }
  } catch {
    // Fallback if ConnectError import fails
  }

  // Default message for non-Connect errors
  const message = error instanceof Error ? error.message : String(error);
  return {
    userMessage: `Ein unerwarteter Fehler ist aufgetreten: ${message}`,
  };
}

export const ErrorBoundary: ParentComponent = (props) => {
  const [error, setError] = createSignal<ErrorInfo | null>(null);

  onError(async (err) => {
    const message = err instanceof Error ? err.message : String(err);
    const stack = err instanceof Error ? err.stack : undefined;
    
    // Check if it's a Connect-RPC error
    const isRpcError = await isConnectError(err);
    const errorDetails = isRpcError 
      ? await getConnectErrorMessage(err)
      : { userMessage: message };

    setError({
      message,
      stack,
      ...errorDetails,
    });
    
    // Dynamically import logger to avoid circular dependencies
    try {
      const { logger } = await import("../lib/logger");
      logger.error("Error caught by boundary", err instanceof Error ? err : new Error(message), {
        component: "ErrorBoundary",
        stack,
        code: errorDetails.code,
        service: errorDetails.service,
        method: errorDetails.method,
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
        <div class="min-h-screen bg-background p-8 flex items-center justify-center">
          <div class="max-w-2xl w-full space-y-6">
            <div class="text-center space-y-2">
              <h1 class="text-3xl font-bold text-destructive">Ein Fehler ist aufgetreten</h1>
              <p class="text-muted-foreground">
                {error()!.userMessage || error()!.message}
              </p>
            </div>

            {/* Show service/method context for RPC errors */}
            {(error()!.service || error()!.method) && (
              <div class="bg-muted p-4 rounded-lg space-y-2">
                <p class="text-sm font-medium">Fehlerdetails:</p>
                <div class="text-sm space-y-1">
                  {error()!.service && (
                    <p>
                      <span class="font-medium">Service:</span>{" "}
                      <code class="bg-background px-2 py-1 rounded">{error()!.service}</code>
                    </p>
                  )}
                  {error()!.method && (
                    <p>
                      <span class="font-medium">Methode:</span>{" "}
                      <code class="bg-background px-2 py-1 rounded">{error()!.method}</code>
                    </p>
                  )}
                  {error()!.code && (
                    <p>
                      <span class="font-medium">Code:</span>{" "}
                      <code class="bg-background px-2 py-1 rounded">{error()!.code}</code>
                    </p>
                  )}
                </div>
              </div>
            )}

            {/* Show stack trace in development */}
            {import.meta.env.DEV && error()!.stack && (
              <details class="bg-muted p-4 rounded-lg">
                <summary class="cursor-pointer font-medium text-sm mb-2">
                  Technische Details (nur in Entwicklung)
                </summary>
                <pre class="text-xs overflow-auto mt-2 bg-background p-3 rounded">
                  {error()!.stack}
                </pre>
              </details>
            )}

            <div class="text-center">
              <button
                onClick={() => window.location.reload()}
                class="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
              >
                Seite neu laden
              </button>
            </div>
          </div>
        </div>
      ) : (
        props.children
      )}
    </>
  );
};
