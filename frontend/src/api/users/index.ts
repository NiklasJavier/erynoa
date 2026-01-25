/**
 * User Service API
 * 
 * Feature-based organization for User Service
 */

// Re-export types
export * from "./types";

// Re-export Connect-RPC client
export { userClient, createAuthenticatedClients } from "../connect/services";
export type { UserClient } from "../connect/services";

// Re-export Connect client class
export { ConnectUsersClient } from "./connect-client";
