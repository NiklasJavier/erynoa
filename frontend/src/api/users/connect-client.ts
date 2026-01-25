/**
 * Connect-RPC Users Client
 * 
 * User operations using Connect-RPC/gRPC-Web
 */

import { createAuthenticatedClients } from "../connect/services";
import { 
  ListUsersRequest,
  GetUserRequest,
} from "../../gen/godstack/v1/user_pb";
import type { User } from "./types";
import { toUser } from "./types";

/**
 * Connect-RPC Users Client
 */
export class ConnectUsersClient {
  private getToken: () => Promise<string | null>;

  constructor(getToken: () => Promise<string | null>) {
    this.getToken = getToken;
  }

  private getClient() {
    const clients = createAuthenticatedClients(this.getToken);
    return clients.users;
  }

  /**
   * List all users
   */
  async list(pageSize: number = 20, pageToken?: string): Promise<User[]> {
    const request = new ListUsersRequest({
      pageSize,
      pageToken,
    });

    const response = await this.getClient().list(request);

    return response.users.map(toUser);
  }

  /**
   * Get a single user by ID
   */
  async get(id: string): Promise<User> {
    const request = new GetUserRequest({ id });

    const response = await this.getClient().get(request);
    const user = response.user;

    if (!user) {
      throw new Error("User not found");
    }

    return toUser(user);
  }

  /**
   * Get current user (from token)
   */
  async getCurrentUser(): Promise<User> {
    // For now, we'll need to extract user ID from token
    // This might require a separate endpoint or token parsing
    // For Connect-RPC, we can add a GetCurrentUser RPC method
    throw new Error("GetCurrentUser not yet implemented via Connect-RPC");
  }
}
