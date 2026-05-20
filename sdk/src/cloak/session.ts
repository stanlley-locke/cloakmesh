import { CloakConfig } from '../types/config';
import { parseAddress } from './address_parser';

export type SessionState = 'idle' | 'connecting' | 'connected' | 'closed';

export class CloakSession {
  private state: SessionState = 'idle';
  readonly pubkey: Uint8Array;

  constructor(
    private readonly address: string,
    private readonly config: CloakConfig,
  ) {
    this.pubkey = parseAddress(address);
  }

  async connect(): Promise<void> {
    this.state = 'connecting';
    // TODO Phase 5.2: fetch descriptor, establish RP, complete 9-act handshake
    this.state = 'connected';
  }

  async send(data: Uint8Array): Promise<void> {
    if (this.state !== 'connected') throw new Error('Session not connected');
    // TODO Phase 5.2: encrypt cell, inject padding, send via gRPC
  }

  async close(): Promise<void> {
    this.state = 'closed';
  }

  getState(): SessionState { return this.state; }
}
