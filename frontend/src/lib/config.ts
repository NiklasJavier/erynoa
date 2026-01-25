/**
 * API Configuration
 * Loaded from backend via Connect-RPC Info Service
 */

import { GetInfoRequest } from "../api/info";
import { logger } from "./logger";
import { validateConfig } from "./config-schema";
import { getApiBaseUrl } from "./api-config";

/**
 * Application Configuration
 * 
 * @deprecated Use Config from "./config-schema" instead
 * Kept for backwards compatibility
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

// Re-export Config type from schema as primary type
export type { Config } from "./config-schema";

/**
 * Fetch application configuration from backend using Connect-RPC
 * This enables dynamic configuration without rebuilding the frontend
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
    const config = {
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
    // Fallback configuration - imported from config-defaults.ts
    // This ensures values stay in sync with backend/config/base.toml
    const { DEFAULT_CONFIG } = await import("./config-defaults");
    return DEFAULT_CONFIG;
  }
}

// Re-export for backwards compatibility
export { getApiBaseUrl };
