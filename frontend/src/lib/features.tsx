/**
 * Feature Flags Context and Hook
 * 
 * Provides easy access to feature flags from application configuration
 * Feature flags are loaded from backend and can be toggled dynamically
 */

import { createContext, useContext, type ParentComponent } from "solid-js";
import type { AppConfig } from "./config";

interface FeatureFlags {
  registration: boolean;
  socialLogin: boolean;
}

interface ConfigContextValue {
  config: () => AppConfig;
  features: () => FeatureFlags;
  isFeatureEnabled: (feature: keyof FeatureFlags) => boolean;
}

const ConfigContext = createContext<ConfigContextValue>();

/**
 * Config Provider
 * Makes application configuration (including feature flags) available to all children
 */
export const ConfigProvider: ParentComponent<{ config: AppConfig }> = (props) => {
  const features = (): FeatureFlags => {
    return props.config.features || {
      registration: false,
      socialLogin: false,
    };
  };

  const isFeatureEnabled = (feature: keyof FeatureFlags): boolean => {
    return features()[feature] ?? false;
  };

  const value: ConfigContextValue = {
    config: () => props.config,
    features,
    isFeatureEnabled,
  };

  return (
    <ConfigContext.Provider value={value}>
      {props.children}
    </ConfigContext.Provider>
  );
};

/**
 * Hook to access application configuration
 * 
 * @throws Error if used outside ConfigProvider
 */
export function useConfig(): ConfigContextValue {
  const context = useContext(ConfigContext);
  if (!context) {
    throw new Error("useConfig must be used within ConfigProvider");
  }
  return context;
}

/**
 * Hook to access feature flags
 * 
 * @returns Object with feature flags and helper function
 * @throws Error if used outside ConfigProvider
 * 
 * @example
 * ```tsx
 * const { features, isFeatureEnabled } = useFeatureFlags();
 * 
 * // Check if registration is enabled
 * if (isFeatureEnabled("registration")) {
 *   // Show registration button
 * }
 * 
 * // Or access directly
 * if (features().socialLogin) {
 *   // Show social login buttons
 * }
 * ```
 */
export function useFeatureFlags() {
  const context = useConfig();
  return {
    features: context.features,
    isFeatureEnabled: context.isFeatureEnabled,
    config: context.config,
  };
}
