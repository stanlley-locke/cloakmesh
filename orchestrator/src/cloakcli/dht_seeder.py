from rich.console import Console
from cloakcli.api.grpc_client import CloakGrpcClient
from proto import cloak_service_pb2
from cloakcli.cloak_protocol import derive_address
import os

console = Console()

def publish_descriptor(address: str, config_path: str):
    # In a real scenario, we'd load the actual pubkey from the config/identity file
    # For this use case implementation, we'll simulate a valid descriptor if not provided
    client = CloakGrpcClient()
    
    console.print(f"Publishing descriptor for [cyan]{address}[/cyan]...")
    
    # Mocking descriptor data for use case demonstration
    descriptor = cloak_service_pb2.CloakDescriptor(
        cloak_address=address,
        identity_pubkey=b"A" * 32, # This would normally come from the node's identity
        version=1,
    )
    
    try:
        response = client.service_stub.PublishDescriptor(descriptor)
        if response.success:
            console.print(f"[green]SUCCESS:[/green] {response.message}")
        else:
            console.print(f"[red]FAILED:[/red] {response.message}")
    except Exception as e:
        console.print(f"[red]ERROR:[/red] {e}")
    finally:
        client.close()

def fetch_descriptor(address: str):
    client = CloakGrpcClient()
    console.print(f"Fetching descriptor for [cyan]{address}[/cyan]...")
    
    request = cloak_service_pb2.DescriptorRequest(cloak_address=address)
    
    try:
        descriptor = client.service_stub.FetchDescriptor(request)
        console.print("[green]SUCCESS: Descriptor found[/green]")
        console.print(f"  Address: {descriptor.cloak_address}")
        console.print(f"  Pubkey:  {descriptor.identity_pubkey.hex()}")
        console.print(f"  Version: {descriptor.version}")
    except Exception as e:
        console.print(f"[red]ERROR:[/red] {e}")
    finally:
        client.close()
