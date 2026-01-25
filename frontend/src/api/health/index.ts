/**
 * Health Service API
 * 
 * Feature-based organization for Health Service
 */

// Re-export types
export * from "./types";

// Re-export Connect-RPC client
export { healthClient } from "../connect/services";
export type { HealthClient } from "../connect/services";
