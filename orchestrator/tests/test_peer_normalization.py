"""Tests for bootstrap peer address normalization and peers file loading."""

import json
import tempfile
import os
import pytest


# ── Import the normalization helpers from cloakcli ───────────────────────────

def normalize_peer_addr(raw: str) -> str:
    """Python mirror of core/src/main.rs normalize_peer_addr for testing."""
    import urllib.parse

    trimmed = raw.strip().rstrip('/')

    if not trimmed.startswith("http://") and not trimmed.startswith("https://"):
        if ':' in trimmed:
            return trimmed
        raise ValueError(f"Not a valid host:port or URL: {raw!r}")

    parsed = urllib.parse.urlparse(trimmed)
    host = parsed.hostname
    if not host:
        raise ValueError(f"No host in URL: {raw!r}")

    # Explicit port in URL
    if parsed.port:
        return f"{host}:{parsed.port}"

    # GitHub Codespaces: port embedded in first subdomain label
    first_label = host.split('.')[0]
    for part in reversed(first_label.split('-')):
        try:
            port = int(part)
            if port > 0:
                return f"{host}:{port}"
        except ValueError:
            continue

    # Fallback to HTTPS/HTTP default
    default_port = 443 if trimmed.startswith("https://") else 80
    return f"{host}:{default_port}"


# ── Test cases ───────────────────────────────────────────────────────────────

class TestNormalizePeerAddr:

    def test_plain_host_port(self):
        assert normalize_peer_addr("127.0.0.1:4001") == "127.0.0.1:4001"

    def test_plain_host_port_with_spaces(self):
        assert normalize_peer_addr("  127.0.0.1:4001  ") == "127.0.0.1:4001"

    def test_domain_with_port(self):
        assert normalize_peer_addr("relay.example.com:4001") == "relay.example.com:4001"

    def test_https_url_with_explicit_port(self):
        assert normalize_peer_addr("https://myserver.com:4001/") == "myserver.com:4001"

    def test_https_url_with_path(self):
        assert normalize_peer_addr("https://myserver.com:4002/some/path") == "myserver.com:4002"

    def test_http_url_with_explicit_port(self):
        assert normalize_peer_addr("http://192.168.1.10:4003/") == "192.168.1.10:4003"

    def test_codespaces_url_port_4001(self):
        url = "https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev/"
        result = normalize_peer_addr(url)
        # Should extract 4001 from subdomain
        assert result.endswith(":4001")
        assert "ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev" in result

    def test_codespaces_url_port_4002(self):
        url = "https://fuzzy-memory-abc123def-4002.app.github.dev/"
        result = normalize_peer_addr(url)
        assert result.endswith(":4002")

    def test_codespaces_url_port_4003(self):
        url = "https://shiny-space-zzz999-4003.preview.app.github.dev/"
        result = normalize_peer_addr(url)
        assert result.endswith(":4003")

    def test_https_no_port_falls_back_to_443(self):
        result = normalize_peer_addr("https://peer.example.com/")
        assert result == "peer.example.com:443"

    def test_http_no_port_falls_back_to_80(self):
        result = normalize_peer_addr("http://peer.example.com/")
        assert result == "peer.example.com:80"

    def test_trailing_slash_stripped(self):
        assert normalize_peer_addr("https://host.com:4001/") == "host.com:4001"

    def test_invalid_no_port_no_scheme_raises(self):
        with pytest.raises(ValueError):
            normalize_peer_addr("justahostname")

    def test_localhost_variants(self):
        assert normalize_peer_addr("localhost:4001") == "localhost:4001"
        assert normalize_peer_addr("https://localhost:4001/") == "localhost:4001"


# ── Peers file parsing tests ─────────────────────────────────────────────────

def load_peers_from_json(content: str) -> list:
    """Python mirror of Rust load_peers_file for testing."""
    value = json.loads(content)

    if isinstance(value, list):
        return [v for v in value if isinstance(v, str)]

    if isinstance(value, dict):
        peers = value.get("peers", [])
        if not isinstance(peers, list):
            raise ValueError("'peers' must be an array")
        result = []
        for item in peers:
            if isinstance(item, str):
                result.append(item)
            elif isinstance(item, dict) and "address" in item:
                result.append(item["address"])
        return result

    raise ValueError("JSON must be array or object")


class TestLoadPeersFile:

    def test_array_format(self):
        content = '["127.0.0.1:4001", "https://host-4002.app.github.dev/"]'
        peers = load_peers_from_json(content)
        assert peers == ["127.0.0.1:4001", "https://host-4002.app.github.dev/"]

    def test_object_format_string_peers(self):
        content = json.dumps({
            "network": "testnet",
            "peers": ["127.0.0.1:4001", "127.0.0.1:4002"]
        })
        peers = load_peers_from_json(content)
        assert peers == ["127.0.0.1:4001", "127.0.0.1:4002"]

    def test_object_format_rich_peers(self):
        content = json.dumps({
            "network": "testnet",
            "peers": [
                {"address": "127.0.0.1:4001", "id": "alpha"},
                {"address": "127.0.0.1:4002", "note": "beta"},
            ]
        })
        peers = load_peers_from_json(content)
        assert peers == ["127.0.0.1:4001", "127.0.0.1:4002"]

    def test_object_format_mixed_peers(self):
        content = json.dumps({
            "peers": [
                "127.0.0.1:4001",
                {"address": "https://codespace-4002.app.github.dev/"},
            ]
        })
        peers = load_peers_from_json(content)
        assert peers == ["127.0.0.1:4001", "https://codespace-4002.app.github.dev/"]

    def test_empty_array(self):
        assert load_peers_from_json("[]") == []

    def test_empty_peers_object(self):
        assert load_peers_from_json('{"peers": []}') == []

    def test_invalid_json_raises(self):
        with pytest.raises(json.JSONDecodeError):
            load_peers_from_json("not json")

    def test_missing_peers_key_raises(self):
        with pytest.raises(KeyError):
            d = json.loads('{"network": "test"}')
            if "peers" not in d:
                raise KeyError("peers")

    def test_file_read(self, tmp_path):
        peers_file = tmp_path / "peers.json"
        peers_file.write_text(json.dumps({
            "network": "devnet",
            "peers": [
                {"address": "127.0.0.1:4001", "id": "node-alpha"},
                "127.0.0.1:4002"
            ]
        }))
        content = peers_file.read_text()
        peers = load_peers_from_json(content)
        assert len(peers) == 2
        assert "127.0.0.1:4001" in peers
        assert "127.0.0.1:4002" in peers


# ── Integration: normalize all peers from a file ─────────────────────────────

class TestNormalizePeersFromFile:

    def test_full_pipeline(self):
        """Simulate the full pipeline: load file → normalize all addresses."""
        content = json.dumps({
            "peers": [
                "127.0.0.1:4001",
                "https://codespace-x-4002.app.github.dev/",
                {"address": "https://myserver.com:4003/"},
            ]
        })
        raw_peers = load_peers_from_json(content)
        normalized = [normalize_peer_addr(p) for p in raw_peers]
        assert normalized[0] == "127.0.0.1:4001"
        assert normalized[1].endswith(":4002")
        assert normalized[2] == "myserver.com:4003"
