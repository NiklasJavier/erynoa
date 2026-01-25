/**
 * Error Types - Harmonisiert mit Backend
 * 
 * Diese Typen entsprechen den Backend ApiError Varianten
 * für konsistente Fehlerbehandlung zwischen Frontend und Backend
 */

/**
 * API Error Response Format
 * Entspricht dem Backend ErrorResponse Format
 */
export interface ApiErrorResponse {
  error: ErrorDetails;
}

/**
 * Error Details
 * Entspricht dem Backend ErrorDetails Format
 */
export interface ErrorDetails {
  code: ErrorCode;
  message: string;
  details?: unknown;
}

/**
 * Error Codes - Harmonisiert mit Backend
 * Entspricht den Backend ApiError::error_code() Werten
 */
export type ErrorCode =
  | "UNAUTHORIZED"
  | "FORBIDDEN"
  | "INVALID_TOKEN"
  | "VALIDATION_ERROR"
  | "BAD_REQUEST"
  | "NOT_FOUND"
  | "CONFLICT"
  | "DATABASE_ERROR"
  | "CACHE_ERROR"
  | "INTERNAL_ERROR"
  | "SERVICE_UNAVAILABLE";

/**
 * Legacy ApiError Interface (für Rückwärtskompatibilität)
 * @deprecated Verwende ApiErrorResponse stattdessen
 * Nur für toLegacyError() Helper verwendet
 */
export interface LegacyApiError {
  status: number;
  message: string;
  code?: string;
}

/**
 * Konvertiert eine ApiErrorResponse zu einem LegacyApiError (für Kompatibilität)
 * @deprecated Verwende ApiErrorResponse direkt
 */
export function toLegacyError(response: ApiErrorResponse): LegacyApiError {
  const statusMap: Record<ErrorCode, number> = {
    UNAUTHORIZED: 401,
    FORBIDDEN: 403,
    INVALID_TOKEN: 401,
    VALIDATION_ERROR: 400,
    BAD_REQUEST: 400,
    NOT_FOUND: 404,
    CONFLICT: 409,
    DATABASE_ERROR: 500,
    CACHE_ERROR: 500,
    INTERNAL_ERROR: 500,
    SERVICE_UNAVAILABLE: 503,
  };

  return {
    status: statusMap[response.error.code] || 500,
    message: response.error.message,
    code: response.error.code,
  };
}

/**
 * Prüft ob ein Error ein bestimmter ErrorCode ist
 */
export function isErrorCode(error: unknown, code: ErrorCode): boolean {
  if (typeof error === "object" && error !== null) {
    const apiError = error as ApiErrorResponse;
    return apiError.error?.code === code;
  }
  return false;
}

/**
 * Extrahiert Error-Details aus einem Error
 */
export function extractError(error: unknown): ErrorDetails | null {
  if (typeof error === "object" && error !== null) {
    const apiError = error as ApiErrorResponse;
    if (apiError.error) {
      return apiError.error;
    }
  }
  return null;
}
