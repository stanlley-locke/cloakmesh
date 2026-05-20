from cloakcli.auth_generator import issue_token
from cloakcli.cloak_protocol import derive_address
import json


def test_issue_token_fields():
    token_json = issue_token("test.cloak", "read", 3600)
    token = json.loads(token_json)
    assert token["cloak_address"] == "test.cloak"
    assert token["scope"] == "read"
    assert "token_id" in token
    assert token["expires_at"] > 0
