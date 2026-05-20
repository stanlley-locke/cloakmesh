import { enum as ZodEnum } from 'zod';

export enum CloakErrorCode {
    // 1000: CRYPTO
    KEY_GEN_FAILED = 1001,
    SIG_VERIFY_FAILED = 1002,
    AEAD_DECRYPT_FAILED = 1003,
    HANDSHAKE_INCOMPLETE = 1004,
    NONCE_EXHAUSTED = 1005,

    // 2000: PROTOCOL
    INVALID_ADDRESS = 2001,
    CHECKSUM_MISMATCH = 2002,
    OVERSIZED_PAYLOAD = 2003,

    // 3000: ROUTING
    DHT_KEY_NOT_FOUND = 3001,
    CIRCUIT_BUILD_FAILED = 3002,
    INSUFFICIENT_RELAYS = 3003,

    // 4000: NETWORK
    CONNECTION_REFUSED = 4001,
    TIMEOUT = 4002,
    GRPC_INTERNAL = 4003,

    // 5000: AUTH
    TOKEN_EXPIRED = 5001,
    INSUFFICIENT_SCOPE = 5002,
    ZK_PROOF_INVALID = 5003,

    UNKNOWN = 9999,
}

export class CloakError extends Error {
    public readonly code: CloakErrorCode;
    public readonly timestamp: number;

    constructor(code: CloakErrorCode, message: string) {
        super(`[${code}] ${message}`);
        this.code = code;
        this.name = 'CloakError';
        this.timestamp = Date.now();
        Object.setPrototypeOf(this, CloakError.prototype);
    }
}

export class NetworkError extends CloakError {
    constructor(message: string) {
        super(CloakErrorCode.CONNECTION_REFUSED, message);
    }
}

export class AuthError extends CloakError {
    constructor(message: string) {
        super(CloakErrorCode.INSUFFICIENT_SCOPE, message);
    }
}

export class ProtocolError extends CloakError {
    constructor(message: string) {
        super(CloakErrorCode.INVALID_ADDRESS, message);
    }
}
