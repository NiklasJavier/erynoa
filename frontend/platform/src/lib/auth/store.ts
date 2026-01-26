/**
 * Auth Store für Svelte 5
 * 
 * Reaktiver Auth-State mit Svelte 5 Runes
 */

import { writable, derived, type Readable } from 'svelte/store';
import type { User } from 'oidc-client-ts';
import { browser } from '$app/environment';
import {
  initAuth,
  getAuth,
  login as oidcLogin,
  logout as oidcLogout,
  handleCallback as oidcHandleCallback,
  getUser,
  getAccessToken as oidcGetAccessToken,
} from './oidc';
import { fetchConfig } from '$lib/config';

// State Types
interface AuthStoreState {
  user: User | null;
  isLoading: boolean;
  isInitialized: boolean;
  error: string | null;
}

// Initial State
const initialState: AuthStoreState = {
  user: null,
  isLoading: true,
  isInitialized: false,
  error: null,
};

// Create Store
function createAuthStore() {
  const { subscribe, set, update } = writable<AuthStoreState>(initialState);

  return {
    subscribe,
    
    /**
     * Initialisiere Auth mit Konfiguration vom Backend
     */
    async init(): Promise<void> {
      if (!browser) return;
      
      // Prevent multiple init calls
      let state: AuthStoreState;
      const unsubscribe = authStore.subscribe(s => state = s);
      unsubscribe();
      if (state!.isInitialized) {
        console.log('[AuthStore] Already initialized, skipping');
        return;
      }
      
      update(s => ({ ...s, isLoading: true, error: null }));
      
      try {
        console.log('[AuthStore] Starting init...');
        
        // Lade Config vom Backend mit Timeout (force reload to get latest clientId)
        console.log('[AuthStore] Fetching config from backend...');
        const configPromise = fetchConfig(true); // Force reload to get latest clientId
        const timeoutPromise = new Promise<never>((_, reject) => 
          setTimeout(() => reject(new Error('Config fetch timeout after 5s')), 5000)
        );
        
        const config = await Promise.race([configPromise, timeoutPromise]);
        console.log('[AuthStore] Config loaded:', {
          environment: config.environment,
          clientId: config.auth.clientId,
          issuer: config.auth.issuer,
          platformUrl: config.urls.platform
        });
        
        // Initialisiere OIDC mit Client-ID und Platform-URL vom Backend
        // Verwende Platform-URL aus Config für exakte Redirect-URI-Übereinstimmung mit Zitadel
        initAuth(config.auth.issuer, config.auth.clientId, config.urls.platform);
        console.log('[AuthStore] OIDC initialized with clientId:', config.auth.clientId, 'redirectUri:', `${config.urls.platform}/callback`);
        
        // Prüfe ob User bereits eingeloggt
        let user = null;
        try {
          const userPromise = getUser();
          const userTimeoutPromise = new Promise<never>((_, reject) => 
            setTimeout(() => reject(new Error('User check timeout after 3s')), 3000)
          );
          user = await Promise.race([userPromise, userTimeoutPromise]);
          console.log('[AuthStore] User check complete:', user?.profile?.preferred_username || 'none');
        } catch (userError) {
          console.warn('[AuthStore] User check failed (normal if not logged in):', userError);
        }
        
        update(s => ({
          ...s,
          user,
          isLoading: false,
          isInitialized: true,
        }));
        
        console.log('[AuthStore] Initialized successfully');
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Auth initialization failed';
        console.error('[AuthStore] Init error:', error);
        update(s => ({
          ...s,
          isLoading: false,
          isInitialized: true,
          error: message,
        }));
      }
    },
    
    /**
     * Login starten
     */
    async login(): Promise<void> {
      update(s => ({ ...s, isLoading: true, error: null }));
      try {
        await oidcLogin();
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Login failed';
        update(s => ({ ...s, isLoading: false, error: message }));
        throw error;
      }
    },
    
    /**
     * Logout
     */
    async logout(): Promise<void> {
      update(s => ({ ...s, isLoading: true, error: null }));
      try {
        await oidcLogout();
        set({ ...initialState, isInitialized: true, isLoading: false });
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Logout failed';
        update(s => ({ ...s, isLoading: false, error: message }));
        throw error;
      }
    },
    
    /**
     * Callback verarbeiten
     */
    async handleCallback(): Promise<User> {
      update(s => ({ ...s, isLoading: true, error: null }));
      try {
        // Ensure OIDC is initialized before handling callback
        // Always reload config to ensure we have the latest clientId
        console.log('[AuthStore] Loading config for callback (force reload)...');
        const config = await fetchConfig(true); // Force reload
        console.log('[AuthStore] Config loaded for callback:', {
          clientId: config.auth.clientId,
          issuer: config.auth.issuer,
          platformUrl: config.urls.platform
        });
        // Verwende Platform-URL aus Config für exakte Redirect-URI-Übereinstimmung mit Zitadel
        initAuth(config.auth.issuer, config.auth.clientId, config.urls.platform);
        console.log('[AuthStore] OIDC initialized for callback with clientId:', config.auth.clientId, 'redirectUri:', `${config.urls.platform}/callback`);
        
        const user = await oidcHandleCallback();
        console.log('[AuthStore] Callback processed, user:', {
          username: user?.profile?.preferred_username,
          expired: user?.expired,
          expires_at: user?.expires_at,
          access_token: user?.access_token ? 'present' : 'missing'
        });
        
        // Verify user was set correctly BEFORE updating state
        if (!user) {
          throw new Error('No user returned from callback');
        }
        
        // Check if user is expired (should not be after fresh callback)
        if (user.expired) {
          console.warn('[AuthStore] User is expired after callback, this should not happen');
          // Try to refresh the user
          try {
            const refreshedUser = await getUser();
            if (refreshedUser && !refreshedUser.expired) {
              console.log('[AuthStore] User refreshed successfully');
              update(s => ({
                ...s,
                user: refreshedUser,
                isLoading: false,
                isInitialized: true,
              }));
              return refreshedUser;
            }
          } catch (refreshError) {
            console.error('[AuthStore] Failed to refresh user:', refreshError);
          }
          throw new Error('User expired after callback');
        }
        
        // Update state with user
        update(s => ({
          ...s,
          user,
          isLoading: false,
          isInitialized: true,
        }));
        
        console.log('[AuthStore] User state updated successfully, isAuthenticated should be true now');
        return user;
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Callback failed';
        console.error('[AuthStore] Callback error:', error);
        update(s => ({ ...s, isLoading: false, error: message }));
        throw error;
      }
    },
    
    /**
     * Access Token für API Calls holen
     */
    getAccessToken: oidcGetAccessToken,
    
    /**
     * User Refresh
     */
    async refresh(): Promise<void> {
      const user = await getUser();
      update(s => ({ ...s, user }));
    },
    
    /**
     * Error clearen
     */
    clearError(): void {
      update(s => ({ ...s, error: null }));
    },
  };
}

// Export Singleton Store
export const authStore = createAuthStore();

// Derived Stores für einfacheren Zugriff
export const user: Readable<User | null> = derived(authStore, $auth => $auth.user);
// isAuthenticated: User muss existieren und nicht expired sein
// Wichtig: Prüfe auch, ob der User einen Access Token hat
export const isAuthenticated: Readable<boolean> = derived(
  authStore, 
  $auth => {
    const hasUser = !!$auth.user;
    const notExpired = $auth.user ? !$auth.user.expired : false;
    const hasToken = $auth.user ? !!$auth.user.access_token : false;
    const result = hasUser && notExpired && hasToken;
    
    // Debug logging
    if (hasUser && !result) {
      console.log('[Auth] User exists but not authenticated:', {
        hasUser,
        notExpired,
        hasToken,
        expired: $auth.user?.expired,
        expires_at: $auth.user?.expires_at
      });
    }
    
    return result;
  }
);
export const isLoading: Readable<boolean> = derived(authStore, $auth => $auth.isLoading);
export const authError: Readable<string | null> = derived(authStore, $auth => $auth.error);
