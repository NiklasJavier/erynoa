/**
 * REST API Client
 * HTTP client for backend communication with auth integration
 */

import { getApiBaseUrl } from "../../lib/api-config";
import type { ApiErrorResponse } from "../types/errors";

export class RestClient {
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
        // Try to parse structured error response (harmonized with backend)
        try {
          const jsonResponse = await response.json();
          
          // Check if it's the harmonized error format
          if (jsonResponse.error && jsonResponse.error.code) {
            // Return structured error response (ApiErrorResponse)
            throw jsonResponse as ApiErrorResponse;
          }
          
          // Fallback: Create ApiErrorResponse from legacy format
          const errorResponse: ApiErrorResponse = {
            error: {
              code: (jsonResponse.code || "INTERNAL_ERROR") as ErrorCode,
              message: jsonResponse.message || jsonResponse.error?.message || response.statusText,
              details: jsonResponse.details || jsonResponse.error?.details,
            },
          };
          throw errorResponse;
        } catch (parseError) {
          // If JSON parsing fails or it's not an error response, create ApiErrorResponse
          if (parseError && typeof parseError === "object" && "error" in parseError) {
            // Already an ApiErrorResponse
            throw parseError;
          }
          // Create ApiErrorResponse from status text
          const errorResponse: ApiErrorResponse = {
            error: {
              code: "INTERNAL_ERROR",
              message: response.statusText || "Unknown error",
            },
          };
          throw errorResponse;
        }
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

// Singleton instance
let restClient: RestClient | null = null;

export function initRestClient(getToken: () => Promise<string | null>) {
  restClient = new RestClient(getApiBaseUrl(), getToken);
  return restClient;
}

export function getRestClient(): RestClient {
  if (!restClient) {
    // Return a client without auth for initial config fetch
    return new RestClient(getApiBaseUrl(), async () => null);
  }
  return restClient;
}
