export default async function init(): Promise<void> {
  return Promise.resolve();
}

export class KeyPair {
  private pk: Uint8Array;
  constructor() {
    this.pk = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      this.pk[i] = i;
    }
  }
  public_key(): Uint8Array {
    return this.pk;
  }
}

export class X25519Exchange {
  private pk: Uint8Array;
  constructor() {
    this.pk = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      this.pk[i] = i + 10;
    }
  }
  public_key(): Uint8Array {
    return this.pk;
  }
  diffie_hellman(other_pk: Uint8Array): Uint8Array {
    const ss = new Uint8Array(32);
    ss.fill(42);
    return ss;
  }
}

export function cloak_address_from_pubkey(pubkey: Uint8Array): string {
  return "dummy_address.cloak";
}
