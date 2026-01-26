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
        
        // Lade Config vom Backend mit Timeout
        console.log('[AuthStore] Fetching config...');
        const configPromise = fetchConfig();
        const timeoutPromise = new Promise<never>((_, reject) => 
          setTimeout(() => reject(new Error('Config fetch timeout after 5s')), 5000)
        );
        
        const config = await Promise.race([configPromise, timeoutPromise]);
        console.log('[AuthStore] Config loaded:', config.environment);
        
        // Initialisiere OIDC
        initAuth(config.auth.issuer, config.auth.clientId);
        console.log('[AuthStore] OIDC initialized');
        
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
        if (!getAuth()) {
          console.log('[AuthStore] OIDC not initialized, loading config...');
          const config = await fetchConfig();
          initAuth(config.auth.issuer, config.auth.clientId);
          console.log('[AuthStore] OIDC initialized for callback');
        }
        
        const user = await oidcHandleCallback();
        update(s => ({
          ...s,
          user,
          isLoading: false,
          isInitialized: true,
        }));
        return user;
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Callback failed';
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
export const isAuthenticated: Readable<boolean> = derived(authStore, $auth => !!$auth.user && !$auth.user.expired);
export const isLoading: Readable<boolean> = derived(authStore, $auth => $auth.isLoading);
export const authError: Readable<string | null> = derived(authStore, $auth => $auth.error);
