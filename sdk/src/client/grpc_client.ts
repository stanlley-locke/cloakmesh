import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import path from 'path';
import { ProtoGrpcType } from '../proto/cloakmesh';
import { CloakMeshNodeClient } from '../proto/cloakmesh/v1/CloakMeshNode';
import { CloakServiceClient } from '../proto/cloakmesh/v1/CloakService';

export class CloakClient {
    private nodeClient: CloakMeshNodeClient;
    private serviceClient: CloakServiceClient;

    constructor(host: string = '127.0.0.1', port: number = 4001) {
        const packageDefinition = protoLoader.loadSync(
            [
                path.join(__dirname, '../../../proto/v1/cloakmesh.proto'),
                path.join(__dirname, '../../../proto/v1/cloak_service.proto')
            ],
            {
                keepCase: true,
                longs: String,
                enums: String,
                defaults: true,
                oneofs: true
            }
        );
        const proto = (grpc.loadPackageDefinition(packageDefinition) as any);
        
        const target = `${host}:${port}`;
        this.nodeClient = new proto.cloakmesh.v1.CloakMeshNode(target, grpc.credentials.createInsecure());
        this.serviceClient = new proto.cloakmesh.v1.CloakService(target, grpc.credentials.createInsecure());
    }

    public async ping(nonce: number = 123): Promise<any> {
        return new Promise((resolve, reject) => {
            this.nodeClient.KeepAlive({ nonce, sentAt: { seconds: Math.floor(Date.now() / 1000), nanos: 0 } }, (err, response) => {
                if (err) {
                    reject(err);
                } else {
                    resolve(response);
                }
            });
        });
    }

    public async publishDescriptor(descriptor: any): Promise<any> {
        return new Promise((resolve, reject) => {
            this.serviceClient.PublishDescriptor(descriptor, (err, response) => {
                if (err) {
                    reject(err);
                } else {
                    resolve(response);
                }
            });
        });
    }

    public async fetchDescriptor(address: string): Promise<any> {
        return new Promise((resolve, reject) => {
            this.serviceClient.FetchDescriptor({ cloakAddress: address }, (err, response) => {
                if (err) {
                    reject(err);
                } else {
                    resolve(response);
                }
            });
        });
    }

    public close() {
        this.nodeClient.close();
        this.serviceClient.close();
    }
}
