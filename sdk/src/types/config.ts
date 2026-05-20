import { z } from 'zod';

export const CloakConfigSchema = z.object({
  nodeAddress: z.string(),
  port: z.number().default(4001),
  tlsEnabled: z.boolean().default(false),
  timeoutMs: z.number().default(5000),
  retryAttempts: z.number().default(3),
});

export type CloakConfig = z.infer<typeof CloakConfigSchema>;
