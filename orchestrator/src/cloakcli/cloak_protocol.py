import hashlib
import base64

def derive_address(pubkey: bytes) -> str:
    """
    Python implementation of .cloak address derivation.
    Matches the Rust implementation: base32(version || pubkey || checksum)
    """
    if len(pubkey) != 32:
        raise ValueError("Public key must be 32 bytes")
        
    version = b'\x01'
    # Compute checksum: first 4 bytes of SHA256(SHA256(version || pubkey))
    first = hashlib.sha256(version + pubkey).digest()
    second = hashlib.sha256(first).digest()
    checksum = second[:4]
    
    payload = version + pubkey + checksum
    # Use RFC4648 base32 without padding, lowercase
    encoded = base64.b32encode(payload).decode('utf-8').rstrip('=').lower()
    return f"{encoded}.cloak"

def parse_address(address: str) -> bytes:
    """
    Decodes a .cloak address and returns the 32-byte Ed25519 public key.
    Verifies version and checksum.
    """
    if not address.endswith(".cloak"):
        raise ValueError("Missing .cloak suffix")
    
    base32_part = address[:-6].upper()
    # Add padding back for base64.b32decode
    padding = len(base32_part) % 8
    if padding != 0:
        base32_part += "=" * (8 - padding)
    
    try:
        decoded = base64.b32decode(base32_part)
    except Exception as e:
        raise ValueError(f"Invalid base32: {e}")
    
    if len(decoded) != 37: # 1 (version) + 32 (pubkey) + 4 (checksum)
        raise ValueError(f"Invalid payload length: {len(decoded)}")
        
    version = decoded[0]
    if version != 1:
        raise ValueError(f"Unsupported version: {version}")
        
    pubkey = decoded[1:33]
    checksum = decoded[33:]
    
    # Verify checksum
    first = hashlib.sha256(decoded[:33]).digest()
    second = hashlib.sha256(first).digest()
    expected_checksum = second[:4]
    
    if checksum != expected_checksum:
        raise ValueError("Checksum mismatch")
        
    return pubkey
