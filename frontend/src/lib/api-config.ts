/**
 * Centralized API Configuration
 * 
 * Single source of truth for API URLs and connection settings
 * Harmonized with backend configuration
 */

/**
 * Get API base URL from environment or default
 * Harmonized with backend config
 */
export function getApiBaseUrl(): string {
  return import.meta.env.VITE_API_URL || "http://localhost:3000";
}

/**
 * API Version constant
 * Single source of truth for API version prefix
 */
export const API_VERSION = "/api/v1";

/**
 * Get API version prefix
 * @deprecated Use API_VERSION constant instead
 */
export function getApiVersion(): string {
  return API_VERSION;
}

/**
 * Get full API URL (base + version)
 */
export function getApiUrl(): string {
  return `${getApiBaseUrl()}${getApiVersion()}`;
}

/**
 * API Configuration
 * Harmonized with backend ApplicationSettings
 */
export interface ApiConfig {
  baseUrl: string;
  version: string;
  fullUrl: string;
  timeout: number;
}

/**
 * Get complete API configuration
 */
export function getApiConfig(): ApiConfig {
  return {
    baseUrl: getApiBaseUrl(),
    version: getApiVersion(),
    fullUrl: getApiUrl(),
    timeout: 30000, // 30 seconds
  };
}

/**
 * Check if running in development mode
 */
export function isDevelopment(): boolean {
  return import.meta.env.DEV || import.meta.env.MODE === "development";
}

/**
 * Check if running in production mode
 */
export function isProduction(): boolean {
  return import.meta.env.PROD || import.meta.env.MODE === "production";
}
