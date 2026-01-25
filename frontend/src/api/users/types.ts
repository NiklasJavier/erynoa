/**
 * User Service Types
 * 
 * Protobuf types as primary source with helper functions
 */

// Re-export Protobuf types as primary source
export {
  User as ProtoUser,
  ListUsersRequest,
  ListUsersResponse,
  GetUserRequest,
  GetUserResponse,
} from "../../gen/godstack/v1/user_pb";

/**
 * Frontend User type (compatible with existing code)
 * Converted from Protobuf User
 */
export interface User {
  id: string;
  email: string | null;
  name: string | null;
  roles: string[];
  created_at?: string;
  updated_at?: string;
}

/**
 * Convert ProtoUser to User
 * Helper function for compatibility with existing code
 */
export function toUser(proto: import("../../gen/godstack/v1/user_pb").User): User {
  return {
    id: proto.id,
    email: proto.email || null,
    name: proto.name || null,
    roles: proto.role ? [proto.role] : [],
    created_at: proto.createdAt
      ? new Date(Number(proto.createdAt.seconds) * 1000 + (proto.createdAt.nanos || 0) / 1_000_000).toISOString()
      : undefined,
    updated_at: proto.updatedAt
      ? new Date(Number(proto.updatedAt.seconds) * 1000 + (proto.updatedAt.nanos || 0) / 1_000_000).toISOString()
      : undefined,
  };
}

/**
 * Convert User to ProtoUser (for requests)
 */
export function fromUser(user: Partial<User>): Partial<import("../../gen/godstack/v1/user_pb").User> {
  return {
    id: user.id || "",
    email: user.email || "",
    name: user.name || "",
    role: user.roles?.[0] || "",
  };
}
