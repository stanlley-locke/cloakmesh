from rich.console import Console
from cloakcli.api.grpc_client import CloakGrpcClient

console = Console()

def check_health(host: str, port: int):
    client = CloakGrpcClient(host, port)
    console.print(f"Checking health of node at {host}:{port}...")
    
    response = client.ping()
    if response:
        console.print(f"[green]SUCCESS:[/green] Node is alive. Received pong with nonce: {response.nonce}")
    else:
        console.print(f"[red]FAILED:[/red] Could not connect to node at {host}:{port}")
    
    client.close()
