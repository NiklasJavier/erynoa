/**
 * REST API Endpoints
 * 
 * Typed API endpoint definitions using the REST client
 */

import type {
  HealthResponse,
  ReadinessResponse,
  InfoResponse,
  User,
} from "../types";
import { getRestClient } from "./client";
import { API_VERSION } from "../../lib/api-config";

// API endpoints
export const api = {
  health: () => getRestClient().get<HealthResponse>(`${API_VERSION}/health`),
  ready: () => getRestClient().get<ReadinessResponse>(`${API_VERSION}/ready`),
  info: () => getRestClient().get<InfoResponse>(`${API_VERSION}/info`),
  users: {
    list: () => getRestClient().get<User[]>(`${API_VERSION}/users`),
    get: (id: string) => getRestClient().get<User>(`${API_VERSION}/users/${id}`),
    create: (data: Partial<User>) => getRestClient().post<User>(`${API_VERSION}/users`, data),
    update: (id: string, data: Partial<User>) => getRestClient().put<User>(`${API_VERSION}/users/${id}`, data),
    delete: (id: string) => getRestClient().delete<void>(`${API_VERSION}/users/${id}`),
  },
};
