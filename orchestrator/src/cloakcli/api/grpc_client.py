import grpc
import sys
import os
import time
from google.protobuf.timestamp_pb2 import Timestamp

# Add the generated proto directory to the path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))

from proto import cloakmesh_pb2  # noqa: E402
from proto import cloakmesh_pb2_grpc  # noqa: E402
from proto import cloak_service_pb2  # noqa: E402
from proto import cloak_service_pb2_grpc  # noqa: E402
from proto import telemetry_pb2  # noqa: E402
from proto import telemetry_pb2_grpc  # noqa: E402

class CloakGrpcClient:
    def __init__(self, host="127.0.0.1", port=4001):
        self.channel = grpc.insecure_channel(f"{host}:{port}")
        self.node_stub = cloakmesh_pb2_grpc.CloakMeshNodeStub(self.channel)
        self.service_stub = cloak_service_pb2_grpc.CloakServiceStub(self.channel)
        self.telemetry_stub = telemetry_pb2_grpc.TelemetryServiceStub(self.channel)

    def ping(self, nonce=123):
        try:
            ts = Timestamp()
            ts.FromSeconds(int(time.time()))
            request = cloakmesh_pb2.Ping(nonce=nonce, sent_at=ts)
            response = self.node_stub.KeepAlive(request)
            return response
        except grpc.RpcError as e:
            print(f"gRPC error: {e}")
            return None

    def chat_stream(self, message_iterator):
        try:
            return self.node_stub.ChatStream(message_iterator)
        except grpc.RpcError as e:
            print(f"Chat stream error: {e}")
            return []

    def file_transfer(self, chunk_iterator):
        try:
            return self.node_stub.FileTransfer(chunk_iterator)
        except grpc.RpcError as e:
            print(f"File transfer error: {e}")
            return None

    def host_site(self, address: str, port: int):
        try:
            request = cloak_service_pb2.HostRequest(cloak_address=address, local_port=port)
            return self.service_stub.HostSite(request)
        except grpc.RpcError as e:
            print(f"Host site error: {e}")
            return None

    def get_metrics(self, node_id: str = ""):
        try:
            request = telemetry_pb2.MetricsRequest(node_id=node_id)
            return self.telemetry_stub.GetMetrics(request)
        except grpc.RpcError as e:
            print(f"Get metrics error: {e}")
            return None

    def get_circuits(self, node_id: str = ""):
        try:
            request = telemetry_pb2.MetricsRequest(node_id=node_id)
            return self.telemetry_stub.GetCircuits(request)
        except grpc.RpcError as e:
            print(f"Get circuits error: {e}")
            return None

    def get_relays(self, node_id: str = ""):
        try:
            request = telemetry_pb2.MetricsRequest(node_id=node_id)
            return self.telemetry_stub.GetRelays(request)
        except grpc.RpcError as e:
            print(f"Get relays error: {e}")
            return None

    def publish_descriptor(self, address: str):
        try:
            request = cloak_service_pb2.CloakDescriptor(
                cloak_address=address,
                identity_pubkey=bytes([0] * 32),
                version=1
            )
            return self.service_stub.PublishDescriptor(request)
        except grpc.RpcError as e:
            print(f"Publish descriptor error: {e}")
            return None

    def fetch_descriptor(self, address: str):
        try:
            request = cloak_service_pb2.DescriptorRequest(cloak_address=address)
            return self.service_stub.FetchDescriptor(request)
        except grpc.RpcError as e:
            print(f"Fetch descriptor error: {e}")
            return None

    def close(self):
        self.channel.close()
