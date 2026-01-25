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

// In development, Frontend spricht direkt mit dem Backend an
// In production, same origin is used
const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3000";

/**
 * Fetch application configuration from backend
 * This enables dynamic configuration without rebuilding the frontend
 */
export async function fetchConfig(): Promise<AppConfig> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/v1/info`);
    if (!response.ok) {
      throw new Error(`Failed to fetch config: ${response.status}`);
    }
    const data: BackendInfoResponse = await response.json();
    
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
    // Fallback configuration for development
    return {
      environment: "local",
      version: "0.1.0",
      auth: {
        issuer: "http://localhost:8080",
        clientId: "godstack-frontend",
      },
      urls: {
        frontend: "http://localhost:5173",
        api: "http://localhost:3000",
      },
      features: {
        registration: true,
        socialLogin: false,
      },
    };
  }
}

export { API_BASE_URL };
