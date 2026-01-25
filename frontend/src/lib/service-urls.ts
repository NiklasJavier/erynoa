/**
 * Service URL Constants
 * 
 * Zentrale Definition aller Service-URLs
 * Harmonized with backend/config/base.toml and docs/development/SERVICE_CONFIG.md
 * 
 * @see backend/config/base.toml für Standard-Werte
 * @see docs/development/SERVICE_CONFIG.md für Service-Definitionen
 */

/**
 * Service URLs for local development
 * These values should match backend/config/base.toml
 */
export const SERVICE_URLS = {
  // Application Services
  frontend: "http://localhost:5173",
  api: "http://localhost:3000",
  
  // External Services
  zitadel: "http://localhost:8080",
  zitadelConsole: "http://localhost:8080/ui/console",
  minio: "http://localhost:9000",
  minioConsole: "http://localhost:9001",
  
  // Database & Cache (for display purposes)
  database: "postgresql://localhost:5432",
  cache: "redis://localhost:6379",
} as const;

/**
 * Get service URL by key
 */
export function getServiceUrl(key: keyof typeof SERVICE_URLS): string {
  return SERVICE_URLS[key];
}

/**
 * Get ZITADEL console URL for a specific path
 */
export function getZitadelConsoleUrl(path: string = ""): string {
  const base = SERVICE_URLS.zitadelConsole;
  return path ? `${base}${path.startsWith("/") ? path : `/${path}`}` : base;
}
