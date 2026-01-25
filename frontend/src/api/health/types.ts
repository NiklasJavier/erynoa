/**
 * Health Service Types
 * 
 * Re-exports Protobuf types as primary source
 * Helper functions for type conversion if needed
 */

// Re-export Protobuf types as primary source
export {
  HealthCheckRequest,
  HealthCheckResponse,
  ReadyRequest,
  ReadyResponse,
  ServiceStatus,
} from "../../gen/godstack/v1/health_pb";

// Export ServiceStatus as ProtoServiceStatus for clarity
export type { ServiceStatus as ProtoServiceStatus } from "../../gen/godstack/v1/health_pb";

/**
 * Convert Proto ServiceStatus to our ServiceStatus format
 * (if needed for compatibility)
 */
export function toServiceStatus(proto: import("../../gen/godstack/v1/health_pb").ServiceStatus): {
  status: string;
  latency_ms?: number;
  error?: string;
} {
  return {
    status: proto.healthy ? "online" : "offline",
    latency_ms: proto.latencyMs ? Number(proto.latencyMs) : undefined,
    error: proto.message && !proto.healthy ? proto.message : undefined,
  };
}
