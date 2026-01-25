/**
 * ZITADEL OIDC Authentication
 * Uses oidc-client-ts for PKCE flow (no client secret needed for SPAs)
 */

import { UserManager, User, WebStorageStateStore } from "oidc-client-ts";
import { createSignal, createContext, useContext, type ParentComponent, onMount } from "solid-js";
import { createLogger } from "./logger";

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

  const authLogger = createLogger({ module: "auth" });
  
  authLogger.debug("Creating UserManager with config", {
    authority: issuer,
    client_id: clientId,
    redirect_uri: redirectUri,
    post_logout_redirect_uri: postLogoutRedirectUri,
  });

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
    // PKCE is enabled by default in oidc-client-ts for public clients
    // Explicitly set to ensure compatibility with ZITADEL
    extraQueryParams: {},
    // Ensure metadata is loaded correctly
    metadata: {
      issuer: issuer,
      authorization_endpoint: `${issuer}/oauth/v2/authorize`,
      token_endpoint: `${issuer}/oauth/v2/token`,
      userinfo_endpoint: `${issuer}/oidc/v1/userinfo`,
      end_session_endpoint: `${issuer}/oidc/v1/end_session`,
      jwks_uri: `${issuer}/.well-known/jwks.json`,
    },
  });

  // Handle silent renew errors
  userManager.events.addSilentRenewError((error) => {
      authLogger.error("Silent renew error", error instanceof Error ? error : new Error(String(error)));
  });

  // Handle user loaded events
  userManager.events.addUserLoaded((user) => {
    authLogger.info("User loaded", { username: user?.profile?.preferred_username });
  });

  // Handle user unloaded events
  userManager.events.addUserUnloaded(() => {
    authLogger.info("User unloaded");
  });

  // Handle access token expiring
  userManager.events.addAccessTokenExpiring(() => {
    authLogger.debug("Access token expiring, will renew automatically");
  });

  // Handle access token expired
  userManager.events.addAccessTokenExpired(() => {
    authLogger.warn("Access token expired");
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
    const authLogger = createLogger({ module: "auth" });
    authLogger.info("Initializing Auth", { issuer: props.issuer, clientId: props.clientId });
    
    // Validate auth config
    if (!props.issuer || !props.clientId) {
      authLogger.error("Invalid auth config", undefined, { issuer: props.issuer, clientId: props.clientId });
      setState({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: "Invalid authentication configuration",
      });
      return;
    }
    
    const manager = initAuth(props.issuer, props.clientId);
    authLogger.info("UserManager initialized", {
      authority: manager.settings.authority,
      client_id: manager.settings.client_id,
      redirect_uri: manager.settings.redirect_uri,
    });

    try {
      // Check if we're handling a callback
      if (window.location.pathname === "/callback") {
        authLogger.debug("Processing OIDC callback");
        const user = await manager.signinRedirectCallback();
        authLogger.info("Callback processed successfully", { username: user?.profile?.preferred_username });
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
      authLogger.error("Auth initialization error", error instanceof Error ? error : new Error(String(error)), {
        issuer: props.issuer,
        clientId: props.clientId,
      });
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
    const authLogger = createLogger({ module: "auth" });
    authLogger.debug("Login button clicked");
    
    if (!userManager) {
      authLogger.error("UserManager not initialized", undefined, {
        action: "login",
      });
      setState((s) => ({
        ...s,
        error: "Authentication not initialized. Please refresh the page.",
      }));
      return;
    }
    
    try {
      // Ensure metadata is loaded before redirect
      authLogger.debug("Loading OIDC metadata");
      await userManager.metadataService.getMetadata();
      authLogger.debug("Metadata loaded successfully");
      
      authLogger.info("Starting OIDC redirect", {
        authority: userManager.settings.authority,
        client_id: userManager.settings.client_id,
        redirect_uri: userManager.settings.redirect_uri,
        authorization_endpoint: userManager.metadataService.getAuthorizationEndpoint(),
      });
      
      await userManager.signinRedirect();
      authLogger.debug("Redirect initiated successfully");
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "Login failed";
      authLogger.error("Login error", error instanceof Error ? error : new Error(String(error)), {
        action: "login",
        userManagerSettings: userManager ? {
          authority: userManager.settings.authority,
          client_id: userManager.settings.client_id,
          redirect_uri: userManager.settings.redirect_uri,
        } : null,
      });
      setState((s) => ({
        ...s,
        error: errorMessage,
      }));
    }
  };

  const logout = async () => {
    const authLogger = createLogger({ module: "auth" });
    if (!userManager) return;
    try {
      authLogger.debug("Initiating logout");
      await userManager.signoutRedirect();
    } catch (error) {
      authLogger.error("Logout error", error instanceof Error ? error : new Error(String(error)), {
        action: "logout",
      });
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
