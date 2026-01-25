/**
 * ZITADEL OIDC Authentication
 * Uses oidc-client-ts for PKCE flow (no client secret needed for SPAs)
 */

import { UserManager, User, WebStorageStateStore } from "oidc-client-ts";
import { createSignal, createContext, useContext, type ParentComponent, onMount } from "solid-js";

export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

export interface AuthContextValue extends AuthState {
  login: () => Promise<void>;
  logout: () => Promise<void>;
  getAccessToken: () => Promise<string | null>;
}

const AuthContext = createContext<AuthContextValue>();

let userManager: UserManager | null = null;

/**
 * Initialize the OIDC UserManager
 * Called once with config from backend
 */
export function initAuth(issuer: string, clientId: string) {
  const redirectUri = `${window.location.origin}/callback`;
  const postLogoutRedirectUri = window.location.origin;

  userManager = new UserManager({
    authority: issuer,
    client_id: clientId,
    redirect_uri: redirectUri,
    post_logout_redirect_uri: postLogoutRedirectUri,
    response_type: "code",
    scope: "openid profile email",
    automaticSilentRenew: true,
    userStore: new WebStorageStateStore({ store: window.localStorage }),
    // ZITADEL specific
    loadUserInfo: true,
  });

  // Handle silent renew errors
  userManager.events.addSilentRenewError((error) => {
    console.error("Silent renew error:", error);
  });

  return userManager;
}

/**
 * Auth Provider Component
 * Wraps app and provides auth state/methods to all children
 */
export const AuthProvider: ParentComponent<{ issuer: string; clientId: string }> = (props) => {
  const [state, setState] = createSignal<AuthState>({
    user: null,
    isAuthenticated: false,
    isLoading: true,
    error: null,
  });

  onMount(async () => {
    // Initialize UserManager
    const manager = initAuth(props.issuer, props.clientId);

    try {
      // Check if we're handling a callback
      if (window.location.pathname === "/callback") {
        console.log("Processing OIDC callback...");
        const user = await manager.signinRedirectCallback();
        console.log("Callback processed successfully, user:", user?.profile?.preferred_username);
        setState({
          user,
          isAuthenticated: true,
          isLoading: false,
          error: null,
        });
        // Redirect to home after successful login - use location.replace for full navigation
        window.location.replace("/");
        return;
      }

      // Try to get existing user from storage
      const user = await manager.getUser();
      setState({
        user,
        isAuthenticated: !!user && !user.expired,
        isLoading: false,
        error: null,
      });
    } catch (error) {
      console.error("Auth initialization error:", error);
      setState({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: error instanceof Error ? error.message : "Authentication error",
      });
      // Clear URL params on error and redirect home
      if (window.location.pathname === "/callback") {
        window.location.replace("/");
      }
    }
  });

  const login = async () => {
    if (!userManager) return;
    try {
      await userManager.signinRedirect();
    } catch (error) {
      console.error("Login error:", error);
      setState((s) => ({
        ...s,
        error: error instanceof Error ? error.message : "Login failed",
      }));
    }
  };

  const logout = async () => {
    if (!userManager) return;
    try {
      await userManager.signoutRedirect();
    } catch (error) {
      console.error("Logout error:", error);
    }
  };

  const getAccessToken = async (): Promise<string | null> => {
    if (!userManager) return null;
    const user = await userManager.getUser();
    return user?.access_token || null;
  };

  const contextValue: AuthContextValue = {
    get user() { return state().user; },
    get isAuthenticated() { return state().isAuthenticated; },
    get isLoading() { return state().isLoading; },
    get error() { return state().error; },
    login,
    logout,
    getAccessToken,
  };

  return (
    <AuthContext.Provider value={contextValue}>
      {props.children}
    </AuthContext.Provider>
  );
};

/**
 * Hook to access auth state and methods
 */
export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}

/**
 * Get current user's display name
 */
export function getUserDisplayName(user: User | null): string {
  if (!user?.profile) return "User";
  return user.profile.name || user.profile.preferred_username || user.profile.email || "User";
}

/**
 * Get user's roles from ZITADEL token
 */
export function getUserRoles(user: User | null): string[] {
  if (!user?.profile) return [];
  // ZITADEL stores roles in urn:zitadel:iam:org:project:roles claim
  const roles = user.profile["urn:zitadel:iam:org:project:roles"];
  if (typeof roles === "object" && roles !== null) {
    return Object.keys(roles);
  }
  return [];
}
