#[cfg(test)]
mod tests {
    use crate::proto::v1::{PeerIdentity, CloakDescriptor};

    #[test]
    fn test_proto_types() {
        let peer = PeerIdentity {
            ed25519_pubkey: vec![0u8; 32],
            cloak_address: "test.cloak".to_string(),
            version: 1,
        };
        assert_eq!(peer.version, 1);
        
        let descriptor = CloakDescriptor {
            cloak_address: "test.cloak".to_string(),
            identity_pubkey: vec![0u8; 32],
            version: 1,
            ..Default::default()
        };
        assert_eq!(descriptor.version, 1);
    }
}
