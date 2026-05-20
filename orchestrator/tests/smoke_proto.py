import sys
import os

# Add src to path to ensure we can import the generated protos
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "src")))

try:
    from proto import cloakmesh_pb2
    from proto import cloak_service_pb2
    
    def test_proto_instantiation():
        # Test PeerIdentity
        peer = cloakmesh_pb2.PeerIdentity(
            ed25519_pubkey=b"A" * 32,
            cloak_address="cloak1qy8x3m7n2p5v9k4w6j1r0h8t2f4d.cloak",
            version=1
        )
        assert peer.version == 1
        assert len(peer.ed25519_pubkey) == 32
        print("✓ PeerIdentity instantiation successful")

        # Test CloakDescriptor
        descriptor = cloak_service_pb2.CloakDescriptor(
            cloak_address="test.cloak",
            identity_pubkey=b"B" * 32
        )
        assert descriptor.cloak_address == "test.cloak"
        print("✓ CloakDescriptor instantiation successful")

    if __name__ == "__main__":
        test_proto_instantiation()
        print("All Python proto smoke tests passed!")

except ImportError as e:
    print(f"Error importing generated protos: {e}")
    sys.exit(1)
