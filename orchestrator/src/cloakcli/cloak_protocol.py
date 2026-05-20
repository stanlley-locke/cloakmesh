import hashlib
import base64

def derive_address(pubkey: bytes) -> str:
    """
    Python implementation of .cloak address derivation.
    Matches the Rust implementation: base32(version || pubkey || checksum)
    """
    version = b'\x01'
    # Compute checksum: first 4 bytes of SHA256(SHA256(version || pubkey))
    first = hashlib.sha256(version + pubkey).digest()
    second = hashlib.sha256(first).digest()
    checksum = second[:4]
    
    payload = version + pubkey + checksum
    # Use RFC4648 base32 without padding, lowercase
    encoded = base64.b32encode(payload).decode('utf-8').rstrip('=').lower()
    return f"{encoded}.cloak"
