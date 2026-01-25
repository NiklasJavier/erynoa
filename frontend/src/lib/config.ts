/**
 * Application Configuration
 * 
 * Consolidated configuration module with schema, types, defaults, and fetching logic.
 * All configuration-related code is in one place for better maintainability.
 */

import { z } from "zod";
import { GetInfoRequest } from "../api/info";
import { logger } from "./logger";
import { getApiBaseUrl } from "./api-config";

// ─────────────────────────────────────────────────────────────────────────────
// Configuration Schema (Zod)
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Configuration Schema
 * Validates structure and types of application configuration
 */
export const ConfigSchema = z.object({
  environment: z.enum(["development", "staging", "production", "local"]),
  version: z.string().min(1),
  auth: z.object({
    issuer: z.string().url(),
    clientId: z.string().min(1),
  }),
  urls: z.object({
    frontend: z.string().url(),
    api: z.string().url(),
  }),
  features: z.object({
    registration: z.boolean(),
    socialLogin: z.boolean(),
  }).optional(),
});

// ─────────────────────────────────────────────────────────────────────────────
// Type Definitions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Inferred TypeScript type from schema
 */
export type Config = z.infer<typeof ConfigSchema>;

/**
 * Application Configuration
 * 
 * Legacy type kept for backwards compatibility.
 * Use `Config` type instead.
 */
export interface AppConfig {
  environment: string;
  version: string;
  auth: {
    issuer: string;
    clientId: string;
  };
  urls: {
    frontend: string;
    api: string;
  };
  features: {
    registration: boolean;
    socialLogin: boolean;
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// Default Configuration Values
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Default configuration values
 * 
 * Fallback-Werte wenn Backend nicht erreichbar ist.
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
 * @see docs/reference/SERVICE_CONFIG.md für Service-Definitionen
 */
export const DEFAULT_CONFIG: AppConfig = {
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

// ─────────────────────────────────────────────────────────────────────────────
// Validation Functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Validate configuration against schema
 * 
 * @param config - Configuration object to validate
 * @returns Validated configuration or throws error
 */
export function validateConfig(config: unknown): Config {
  return ConfigSchema.parse(config);
}

/**
 * Safe validation that returns result instead of throwing
 * 
 * @param config - Configuration object to validate
 * @returns Validation result with success flag
 */
export function safeValidateConfig(config: unknown): {
  success: boolean;
  data?: Config;
  error?: z.ZodError;
} {
  const result = ConfigSchema.safeParse(config);
  
  if (result.success) {
    return { success: true, data: result.data };
  } else {
    return { success: false, error: result.error };
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Configuration Fetching
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Fetch application configuration from backend using Connect-RPC
 * 
 * This enables dynamic configuration without rebuilding the frontend.
 * Falls back to DEFAULT_CONFIG if backend is unavailable.
 * 
 * @returns Application configuration
 */
export async function fetchConfig(): Promise<AppConfig> {
  try {
    // Dynamically import infoClient to avoid circular dependencies
    const { infoClient } = await import("../api/connect/services");
    const request = new GetInfoRequest({});
    const response = await infoClient.getInfo(request);
    
    logger.info("Config loaded successfully via Connect-RPC", {
      version: response.version,
      environment: response.environment,
    });
    
    // Validate response
    if (!response) {
      logger.error("Empty response from info service");
      throw new Error("Empty response from info service");
    }
    
    // Map backend response to frontend config
    const config: AppConfig = {
      environment: response.environment || "development",
      version: response.version || "0.1.0",
      auth: {
        issuer: response.auth?.issuer || "",
        clientId: response.auth?.clientId || "",
      },
      urls: {
        frontend: response.urls?.frontend || "",
        api: response.urls?.api || "",
      },
      features: {
        registration: response.features?.registration || false,
        socialLogin: response.features?.socialLogin || false,
      },
    };

    // Validate configuration against schema
    try {
      return validateConfig(config) as AppConfig;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.warn("Config validation failed, using unvalidated config", {
        validationError: errorMessage,
      }, error instanceof Error ? error : undefined);
      return config;
    }
  } catch (error) {
    logger.error("Failed to fetch config, using defaults", error instanceof Error ? error : new Error(String(error)));
    // Fallback to default configuration
    return DEFAULT_CONFIG;
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Re-exports
// ─────────────────────────────────────────────────────────────────────────────

// Re-export for backwards compatibility
export { getApiBaseUrl };
