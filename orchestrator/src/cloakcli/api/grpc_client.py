import grpc
import sys
import os

# Add the generated proto directory to the path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

from proto import cloakmesh_pb2
from proto import cloakmesh_pb2_grpc
from proto import cloak_service_pb2
from proto import cloak_service_pb2_grpc

class CloakGrpcClient:
    def __init__(self, host="127.0.0.1", port=4001):
        self.channel = grpc.insecure_channel(f"{host}:{port}")
        self.node_stub = cloakmesh_pb2_grpc.CloakMeshNodeStub(self.channel)
        self.service_stub = cloak_service_pb2_grpc.CloakServiceStub(self.channel)

    def ping(self, nonce=123):
        try:
            request = cloakmesh_pb2.Ping(nonce=nonce)
            response = self.node_stub.KeepAlive(request)
            return response
        except grpc.RpcError as e:
            print(f"gRPC error: {e}")
            return None

    def close(self):
        self.channel.close()
