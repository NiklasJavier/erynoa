/**
 * Default Configuration Values
 * 
 * Fallback-Werte wenn Backend nicht erreichbar ist
 * 
 * ⚠️ WICHTIG: Diese Werte müssen mit backend synchronisiert bleiben!
 * 
 * Synchronisierte Werte:
 * - environment: base.toml [application].environment
 * - version: Cargo.toml [package].version (siehe backend/src/config/version.rs)
 * - auth.issuer: base.toml [auth].issuer
 * - auth.clientId: base.toml [auth].frontend_client_id
 * - urls.frontend: base.toml [application].frontend_url
 * - urls.api: base.toml [application].api_url
 * 
 * @see backend/config/base.toml für Konfiguration
 * @see backend/src/config/version.rs für Version
 * @see docs/development/SERVICE_CONFIG.md für Service-Definitionen
 */

/**
 * Default configuration values
 * Harmonized with backend configuration
 * 
 * @see backend/config/base.toml for source of truth
 * @see backend/src/config/version.rs for version source
 */
export const DEFAULT_CONFIG = {
  environment: "local",  // base.toml: [application].environment
  version: "0.1.0",  // Cargo.toml: [package].version → backend/src/config/version.rs
  auth: {
    issuer: "http://localhost:8080",  // base.toml: [auth].issuer
    // ⚠️ WICHTIG: clientId wird vom Backend geladen!
    // Dieser Wert ist nur ein Fallback und sollte nicht verwendet werden.
    // Wenn Backend nicht erreichbar ist, wird dieser Wert verwendet, aber
    // die echte Client-ID muss vom Backend kommen (aus local.toml).
    clientId: "godstack-frontend",  // base.toml: [auth].frontend_client_id (Fallback)
  },
  urls: {
    frontend: "http://localhost:5173",  // base.toml: [application].frontend_url
    api: "http://localhost:3000",  // base.toml: [application].api_url
  },
  features: {
    registration: true,
    socialLogin: false,
  },
} as const;
