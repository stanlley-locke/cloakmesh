from enum import IntEnum

class CloakErrorCode(IntEnum):
    # 1000: CRYPTO
    KEY_GEN_FAILED = 1001
    SIG_VERIFY_FAILED = 1002
    AEAD_DECRYPT_FAILED = 1003
    HANDSHAKE_INCOMPLETE = 1004
    NONCE_EXHAUSTED = 1005

    # 2000: PROTOCOL
    INVALID_ADDRESS = 2001
    CHECKSUM_MISMATCH = 2002
    OVERSIZED_PAYLOAD = 2003

    # 3000: ROUTING
    DHT_KEY_NOT_FOUND = 3001
    CIRCUIT_BUILD_FAILED = 3002
    INSUFFICIENT_RELAYS = 3003

    # 4000: NETWORK
    CONNECTION_REFUSED = 4001
    TIMEOUT = 4002
    GRPC_INTERNAL = 4003

    # 5000: AUTH
    TOKEN_EXPIRED = 5001
    INSUFFICIENT_SCOPE = 5002
    ZK_PROOF_INVALID = 5003

    UNKNOWN = 9999

class CloakError(Exception):
    def __init__(self, code: CloakErrorCode, message: str):
        self.code = code
        self.message = message
        super().__init__(f"[{code.name} ({code.value})] {message}")

class NetworkError(CloakError):
    def __init__(self, message: str):
        super().__init__(CloakErrorCode.CONNECTION_REFUSED, message)

class ProtocolError(CloakError):
    def __init__(self, message: str):
        super().__init__(CloakErrorCode.INVALID_ADDRESS, message)
