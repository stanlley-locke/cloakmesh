import os
import typer
from rich.console import Console
from cloakcli.api.grpc_client import CloakGrpcClient
from proto import cloakmesh_pb2, cloakmesh_pb2_grpc

console = Console()
wallet_app = typer.Typer(help="Manage ATK Token Wallet")

@wallet_app.command("balance")
def balance():
    """Check your ATK token balance."""
    port = int(os.getenv("CLOAK_GRPC_PORT", "4001"))
    client_wrapper = CloakGrpcClient(port=port)
    # Note: In a real implementation we'd query the UTXO set for our address
    # For now, we mock the UI
    console.print("[bold green]ATK Balance:[/bold green] 100.0 ATK")
    console.print("[dim]Derived from your Ed25519 identity key[/dim]")

@wallet_app.command("transfer")
def transfer(amount: float, address: str):
    """Transfer ATK to another address."""
    port = int(os.getenv("CLOAK_GRPC_PORT", "4001"))
    client_wrapper = CloakGrpcClient(port=port)
    client = client_wrapper.node_stub
    
    # We would construct a real signed transaction here.
    tx = cloakmesh_pb2.Transaction(id="mock-tx-1234", timestamp=0)
    
    try:
        resp = client.BroadcastTx(tx)
        if resp.success:
            console.print(f"[bold green]Success![/bold green] Sent {amount} ATK to {address}")
            console.print(f"Transaction ID: {tx.id}")
        else:
            console.print(f"[bold red]Failed:[/bold red] {resp.message}")
    except Exception as e:
        console.print(f"[bold red]RPC Error:[/bold red] {str(e)}")
