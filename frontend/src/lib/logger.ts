/**
 * Structured Logging Utility
 * 
 * Provides consistent, structured logging across the frontend application.
 * Similar to backend's tracing-based logging for better observability.
 * 
 * Features:
 * - Development/Production mode awareness
 * - Structured metadata support
 * - Error tracking with stack traces
 * - Log levels (debug, info, warn, error)
 * - Extensible error tracking service integration
 */

import { isDevelopment } from "./api-config";

export type LogLevel = "debug" | "info" | "warn" | "error";

export interface LogMeta {
  [key: string]: unknown;
}

/**
 * Error Tracking Service Interface
 * 
 * Can be implemented by error tracking services like Sentry, LogRocket, etc.
 */
interface ErrorTrackingService {
  captureException(error: Error, context?: LogMeta): void;
  captureMessage(message: string, level: LogLevel, context?: LogMeta): void;
}

/**
 * Logger configuration
 */
interface LoggerConfig {
  level: LogLevel;
  enableStackTraces: boolean;
  errorTrackingService?: ErrorTrackingService;
}

/**
 * Default logger configuration
 */
const defaultConfig: LoggerConfig = {
  level: isDevelopment() ? "debug" : "info",
  enableStackTraces: isDevelopment(),
  errorTrackingService: undefined, // Can be set via setErrorTrackingService()
};

/**
 * Log level priority (higher = more important)
 */
const logLevels: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

/**
 * Check if a log level should be logged
 */
function shouldLog(level: LogLevel, config: LoggerConfig): boolean {
  return logLevels[level] >= logLevels[config.level];
}

/**
 * Format log message with metadata
 */
function formatMessage(
  level: LogLevel,
  message: string,
  meta?: LogMeta,
  error?: Error
): string {
  const prefix = `[${level.toUpperCase()}]`;
  const parts = [prefix, message];

  if (meta && Object.keys(meta).length > 0) {
    parts.push(JSON.stringify(meta));
  }

  if (error) {
    parts.push(`Error: ${error.message}`);
    if (error.stack) {
      parts.push(`Stack: ${error.stack}`);
    }
  }

  return parts.join(" ");
}

/**
 * Structured Logger
 * 
 * Provides consistent logging interface similar to backend's tracing
 */
class Logger {
  public config: LoggerConfig;

  constructor(config: Partial<LoggerConfig> = {}) {
    this.config = { ...defaultConfig, ...config };
  }

  /**
   * Debug log (development only)
   * Use for detailed debugging information
   */
  debug(message: string, meta?: LogMeta): void {
    if (!shouldLog("debug", this.config)) return;
    if (!isDevelopment()) return;

    console.debug(formatMessage("debug", message, meta));
  }

  /**
   * Info log
   * Use for general information about application flow
   */
  info(message: string, meta?: LogMeta): void {
    if (!shouldLog("info", this.config)) return;

    console.info(formatMessage("info", message, meta));
  }

  /**
   * Warning log
   * Use for potentially problematic situations
   */
  warn(message: string, meta?: LogMeta, error?: Error): void {
    if (!shouldLog("warn", this.config)) return;

    console.warn(formatMessage("warn", message, meta, error));
  }

  /**
   * Error log
   * Use for error conditions that need attention
   */
  error(message: string, error?: Error, meta?: LogMeta): void {
    if (!shouldLog("error", this.config)) return;

    const formatted = formatMessage("error", message, meta, error);
    console.error(formatted);

    // Send to error tracking service if configured
    if (error && this.config.errorTrackingService) {
      try {
        this.config.errorTrackingService.captureException(error, meta);
      } catch (trackingError) {
        // Don't let error tracking break the app
        console.warn("Error tracking service failed:", trackingError);
      }
    }
  }

  /**
   * Create a child logger with additional context
   * Useful for scoped logging (e.g., per-component or per-module)
   */
  child(context: LogMeta): Logger {
    const childLogger = new Logger(this.config);
    
    // Wrap methods to include context
    const originalDebug = childLogger.debug.bind(childLogger);
    const originalInfo = childLogger.info.bind(childLogger);
    const originalWarn = childLogger.warn.bind(childLogger);
    const originalError = childLogger.error.bind(childLogger);

    childLogger.debug = (message: string, meta?: LogMeta) => {
      originalDebug(message, { ...context, ...meta });
    };

    childLogger.info = (message: string, meta?: LogMeta) => {
      originalInfo(message, { ...context, ...meta });
    };

    childLogger.warn = (message: string, meta?: LogMeta, error?: Error) => {
      originalWarn(message, { ...context, ...meta }, error);
    };

    childLogger.error = (message: string, error?: Error, meta?: LogMeta) => {
      originalError(message, error, { ...context, ...meta });
    };

    return childLogger;
  }
}

/**
 * Default logger instance
 * Use this for general application logging
 */
export const logger = new Logger();

/**
 * Set error tracking service for the default logger
 * 
 * @example
 * ```typescript
 * // With Sentry (when installed)
 * import * as Sentry from "@sentry/browser";
 * setErrorTrackingService({
 *   captureException: (error, context) => {
 *     Sentry.captureException(error, { extra: context });
 *   },
 *   captureMessage: (message, level, context) => {
 *     Sentry.captureMessage(message, { level, extra: context });
 *   },
 * });
 * ```
 */
export function setErrorTrackingService(service: ErrorTrackingService): void {
  logger.config.errorTrackingService = service;
}

/**
 * Create a logger for a specific module/component
 * 
 * @example
 * ```typescript
 * const apiLogger = createLogger({ module: "api" });
 * apiLogger.info("Request sent", { endpoint: "/users" });
 * ```
 */
export function createLogger(context: LogMeta): Logger {
  return logger.child(context);
}

/**
 * Convenience functions for common logging patterns
 */
export const log = {
  /**
   * Log API request
   */
  apiRequest: (method: string, endpoint: string, meta?: LogMeta) => {
    logger.debug("API Request", {
      method,
      endpoint,
      ...meta,
    });
  },

  /**
   * Log API response
   */
  apiResponse: (method: string, endpoint: string, status: number, meta?: LogMeta) => {
    const level = status >= 400 ? "error" : status >= 300 ? "warn" : "debug";
    if (level === "error") {
      logger.error("API Response", undefined, {
        method,
        endpoint,
        status,
        ...meta,
      });
    } else if (level === "warn") {
      logger.warn("API Response", {
        method,
        endpoint,
        status,
        ...meta,
      });
    } else {
      logger.debug("API Response", {
        method,
        endpoint,
        status,
        ...meta,
      });
    }
  },

  /**
   * Log API error
   */
  apiError: (method: string, endpoint: string, error: Error, meta?: LogMeta) => {
    logger.error("API Error", error, {
      method,
      endpoint,
      ...meta,
    });
  },

  /**
   * Log Connect-RPC request
   */
  connectRequest: (service: string, method: string, meta?: LogMeta) => {
    logger.debug("Connect-RPC Request", {
      service,
      method,
      ...meta,
    });
  },

  /**
   * Log Connect-RPC response
   */
  connectResponse: (service: string, method: string, success: boolean, meta?: LogMeta) => {
    if (success) {
      logger.debug("Connect-RPC Response", {
        service,
        method,
        success,
        ...meta,
      });
    } else {
      logger.error("Connect-RPC Response", undefined, {
        service,
        method,
        success,
        ...meta,
      });
    }
  },

  /**
   * Log Connect-RPC error
   */
  connectError: (service: string, method: string, error: Error, meta?: LogMeta) => {
    logger.error("Connect-RPC Error", error, {
      service,
      method,
      ...meta,
    });
  },
};
