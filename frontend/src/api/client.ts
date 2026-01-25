/**
 * API Client
 * HTTP client for backend communication with auth integration
 */

import { API_BASE_URL } from "../lib/config";

export interface ApiError {
  status: number;
  message: string;
  code?: string;
}

export class ApiClient {
  private baseUrl: string;
  private getToken: () => Promise<string | null>;

  constructor(baseUrl: string, getToken: () => Promise<string | null>) {
    this.baseUrl = baseUrl;
    this.getToken = getToken;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const token = await this.getToken();
    
    const headers: HeadersInit = {
      "Content-Type": "application/json",
      ...options.headers,
    };

    if (token) {
      (headers as Record<string, string>)["Authorization"] = `Bearer ${token}`;
    }

    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        ...options,
        headers,
      });

      if (!response.ok) {
        const error: ApiError = {
          status: response.status,
          message: response.statusText,
        };
        
        try {
          const body = await response.json();
          error.message = body.message || body.error || response.statusText;
          error.code = body.code;
        } catch {
          // Ignore JSON parse errors
        }

        throw error;
      }

      // Handle empty responses
      const contentType = response.headers.get("content-type");
      if (contentType?.includes("application/json")) {
        try {
          return await response.json();
        } catch (parseError) {
          console.error("Failed to parse JSON response:", parseError);
          throw new Error(`Failed to parse response: ${parseError instanceof Error ? parseError.message : "Unknown error"}`);
        }
      }
      
      return {} as T;
    } catch (error) {
      console.error(`API request failed for ${endpoint}:`, error);
      throw error;
    }
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "GET" });
  }

  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "DELETE" });
  }
}

// Create a default client instance (to be initialized with auth)
let apiClient: ApiClient | null = null;

export function initApiClient(getToken: () => Promise<string | null>) {
  apiClient = new ApiClient(API_BASE_URL, getToken);
  return apiClient;
}

export function getApiClient(): ApiClient {
  if (!apiClient) {
    // Return a client without auth for initial config fetch
    return new ApiClient(API_BASE_URL, async () => null);
  }
  return apiClient;
}

// Type definitions for API responses
export interface HealthResponse {
  status: string;
}

export interface ReadinessResponse {
  status: string;
  services: {
    database: { status: string; latency_ms: number };
    cache: { status: string; latency_ms: number };
    storage: { status: string; latency_ms: number };
    auth: { status: string };
  };
  uptime_secs: number;
}

export interface InfoResponse {
  name: string;
  version: string;
  environment: string;
  rust_version: string;
}

export interface User {
  id: string;
  email: string | null;
  name: string | null;
  roles: string[];
}

// API endpoints
export const api = {
  health: () => getApiClient().get<HealthResponse>("/api/v1/health"),
  ready: () => getApiClient().get<ReadinessResponse>("/api/v1/ready"),
  info: () => getApiClient().get<InfoResponse>("/api/v1/info"),
  users: {
    list: () => getApiClient().get<User[]>("/api/v1/users"),
    get: (id: string) => getApiClient().get<User>(`/api/v1/users/${id}`),
    create: (data: Partial<User>) => getApiClient().post<User>("/api/v1/users", data),
    update: (id: string, data: Partial<User>) => getApiClient().put<User>(`/api/v1/users/${id}`, data),
    delete: (id: string) => getApiClient().delete<void>(`/api/v1/users/${id}`),
  },
};
