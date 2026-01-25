/**
 * Configuration Schema
 * 
 * Zod schema for validating application configuration
 * Ensures type safety and runtime validation
 */

import { z } from "zod";

/**
 * Configuration Schema
 * Validates structure and types of application configuration
 */
export const ConfigSchema = z.object({
  environment: z.enum(["development", "staging", "production", "local"]),
  version: z.string().min(1),
  auth: z.object({
    issuer: z.string().url(),
    clientId: z.string().min(1),
  }),
  urls: z.object({
    frontend: z.string().url(),
    api: z.string().url(),
  }),
  features: z.object({
    registration: z.boolean(),
    socialLogin: z.boolean(),
  }).optional(),
});

/**
 * Inferred TypeScript type from schema
 */
export type Config = z.infer<typeof ConfigSchema>;

/**
 * Validate configuration against schema
 * 
 * @param config - Configuration object to validate
 * @returns Validated configuration or throws error
 */
export function validateConfig(config: unknown): Config {
  return ConfigSchema.parse(config);
}

/**
 * Safe validation that returns result instead of throwing
 * 
 * @param config - Configuration object to validate
 * @returns Validation result with success flag
 */
export function safeValidateConfig(config: unknown): {
  success: boolean;
  data?: Config;
  error?: z.ZodError;
} {
  const result = ConfigSchema.safeParse(config);
  
  if (result.success) {
    return { success: true, data: result.data };
  } else {
    return { success: false, error: result.error };
  }
}
