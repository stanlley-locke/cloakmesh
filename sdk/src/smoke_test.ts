import { PeerIdentity } from './proto/cloakmesh/v1/PeerIdentity';
import { CloakDescriptor } from './proto/cloakmesh/v1/CloakDescriptor';

function smokeTest() {
    try {
        const peer: PeerIdentity = {
            ed25519Pubkey: Buffer.from('A'.repeat(32)),
            cloakAddress: 'cloak1qy8x3m7n2p5v9k4w6j1r0h8t2f4d.cloak',
            version: 1
        };
        console.log('✓ PeerIdentity type check successful', peer.cloakAddress);

        const descriptor: CloakDescriptor = {
            cloakAddress: 'test.cloak',
            identityPubkey: Buffer.from('B'.repeat(32)),
            version: 1,
            issuedAt: { seconds: Math.floor(Date.now() / 1000), nanos: 0 }
        };
        console.log('✓ CloakDescriptor type check successful', descriptor.cloakAddress);

        console.log('All TypeScript proto smoke tests passed!');
    } catch (error) {
        console.error('TypeScript smoke test failed:', error);
        process.exit(1);
    }
}

smokeTest();
