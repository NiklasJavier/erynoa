/**
 * Info Service API
 * 
 * Feature-based organization for Info Service
 */

// Re-export types
export * from "./types";

// Re-export Connect-RPC client
export { infoClient } from "../connect/services";
export type { InfoClient } from "../connect/services";
