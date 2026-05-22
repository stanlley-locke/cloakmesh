from rich.console import Console
from cloakcli.api.grpc_client import CloakGrpcClient
from proto import cloak_service_pb2
from cloakcli.cloak_protocol import parse_address

console = Console()

def publish_descriptor(address: str, config_path: str):
    client = CloakGrpcClient()
    
    console.print(f"Publishing descriptor for [cyan]{address}[/cyan]...")
    
    try:
        # Extract the pubkey from the address so the core's validation passes
        pubkey = parse_address(address)
        
        descriptor = cloak_service_pb2.CloakDescriptor(
            cloak_address=address,
            identity_pubkey=pubkey,
            version=1,
        )
        
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
