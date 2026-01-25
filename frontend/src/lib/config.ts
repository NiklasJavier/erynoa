/**
 * API Configuration
 * Loaded from backend /api/v1/info endpoint
 */

// Backend response type
interface BackendInfoResponse {
  version: string;
  environment: string;
  auth_issuer: string;
  auth_client_id: string;
  frontend_url: string;
  api_url: string;
}

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

// Import centralized API config
import { getApiBaseUrl, API_VERSION } from "./api-config";

/**
 * Fetch application configuration from backend
 * This enables dynamic configuration without rebuilding the frontend
 */
export async function fetchConfig(): Promise<AppConfig> {
  try {
    const apiUrl = getApiBaseUrl();
    console.log("Fetching config from:", `${apiUrl}${API_VERSION}/info`);
    const response = await fetch(`${apiUrl}${API_VERSION}/info`);
    if (!response.ok) {
      throw new Error(`Failed to fetch config: ${response.status}`);
    }
    const data: BackendInfoResponse = await response.json();
    console.log("Config loaded successfully:", data);
    
    // Map backend response to frontend config
    return {
      environment: data.environment,
      version: data.version,
      auth: {
        issuer: data.auth_issuer,
        clientId: data.auth_client_id,
      },
      urls: {
        frontend: data.frontend_url,
        api: data.api_url,
      },
      features: {
        registration: true,
        socialLogin: false,
      },
    };
  } catch (error) {
    console.error("Failed to fetch config, using defaults:", error);
    // Fallback configuration - imported from config-defaults.ts
    // This ensures values stay in sync with backend/config/base.toml
    const { DEFAULT_CONFIG } = await import("./config-defaults");
    return DEFAULT_CONFIG;
  }
}

// Re-export for backwards compatibility
export { getApiBaseUrl };
