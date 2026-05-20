import { CloakClient } from '../src/client/grpc_client';

describe('CloakClient', () => {
    it('should be instantiable', () => {
        const client = new CloakClient('127.0.0.1', 4001);
        expect(client).toBeDefined();
        client.close();
    });
});
