import secrets
import time
from pydantic import BaseModel


class CapabilityToken(BaseModel):
    token_id: str
    cloak_address: str
    scope: str
    expires_at: int
    signature: str = ""  # TODO Phase 4.3: Ed25519 sign via gRPC


def issue_token(address: str, scope: str, ttl_secs: int) -> str:
    token = CapabilityToken(
        token_id=secrets.token_hex(16),
        cloak_address=address,
        scope=scope,
        expires_at=int(time.time()) + ttl_secs,
    )
    return token.model_dump_json()
