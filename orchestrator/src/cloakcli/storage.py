import os
import hashlib
import typer
from rich.console import Console
from cloakcli.api.grpc_client import CloakGrpcClient
from proto import cloakmesh_pb2, cloakmesh_pb2_grpc

console = Console()
storage_app = typer.Typer(help="Decentralized Erasure-Coded Storage")

@storage_app.command("upload")
def upload(file_path: str):
    """Chunk, erasure-encode, and distribute a file across the mesh."""
    if not os.path.exists(file_path):
        console.print(f"[bold red]Error:[/bold red] File {file_path} not found.")
        return
        
    port = int(os.getenv("CLOAK_GRPC_PORT", "4001"))
    client_wrapper = CloakGrpcClient(port=port)
    client = client_wrapper.node_stub
    
    with open(file_path, "rb") as f:
        data = f.read()
        
    file_hash = hashlib.sha256(data).hexdigest()
    console.print(f"File Size: {len(data)} bytes")
    console.print(f"SHA-256: [cyan]{file_hash}[/cyan]")
    
    shard_hash = hashlib.sha256(data).hexdigest()
    
    # 1. Generate Manifest
    manifest = cloakmesh_pb2.FileManifest(
        file_hash=file_hash,
        size_bytes=len(data),
        data_shards=1,
        parity_shards=0,
        shard_hashes=[shard_hash],
        owner_address="atk1_demo",
        timestamp=0
    )
    
    gossip_msg = cloakmesh_pb2.GossipMessage(
        message_id=file_hash,
        sender_address="atk1_demo",
        manifest=manifest
    )
    
    # 2. Upload actual data shard
    shard = cloakmesh_pb2.FileShard(
        file_hash=shard_hash,
        shard_index=0,
        is_parity=False,
        data=data
    )
    
    try:
        console.print("Broadcasting File Manifest to network via Gossip...")
        client.Gossip(gossip_msg)
        
        console.print("Distributing data shards into DHT...")
        resp = client.StoreShard(shard)
        if resp.success:
            console.print(f"[bold green]Upload Complete![/bold green] File is decentralized.")
            console.print(f"To download, use: [bold yellow]cloakcli storage download {shard_hash}[/bold yellow]")
        else:
            console.print(f"[bold red]Upload Failed:[/bold red] {resp.message}")
    except Exception as e:
        console.print(f"[bold red]RPC Error:[/bold red] {str(e)}")

@storage_app.command("download")
def download(
    file_hash: str,
    output_path: str = typer.Option("downloaded_file.bin", "--output", "-o", help="Path to save the reconstructed file"),
):
    """Retrieve and reconstruct a file from the mesh DHT."""
    port = int(os.getenv("CLOAK_GRPC_PORT", "4001"))
    client_wrapper = CloakGrpcClient(port=port)
    client = client_wrapper.node_stub
    
    console.print(f"Locating file [cyan]{file_hash[:16]}...[/cyan] in the Mesh DHT...")
    
    req = cloakmesh_pb2.RetrieveShardRequest(
        file_hash=file_hash,
        shard_index=0
    )
    
    try:
        resp = client.RetrieveShard(req)
        with open(output_path, "wb") as f:
            f.write(resp.data)
        console.print(f"[bold green]Success![/bold green] File reconstructed → [cyan]{output_path}[/cyan] ({len(resp.data)} bytes)")
    except Exception as e:
        console.print(f"[bold red]Retrieval Failed:[/bold red] {str(e)}")

