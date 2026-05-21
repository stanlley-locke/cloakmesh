from rich.console import Console
from cloakcli.api.grpc_client import CloakGrpcClient
from proto import cloakmesh_pb2
import time
from google.protobuf.timestamp_pb2 import Timestamp

console = Console()

def send_chat_message(text: str, sender: str):
    client = CloakGrpcClient()
    
    def message_generator():
        ts = Timestamp()
        ts.FromSeconds(int(time.time()))
        yield cloakmesh_pb2.ChatMessage(
            sender=sender,
            text=text,
            sent_at=ts
        )

    try:
        console.print(f"[cyan]Sending chat message from '{sender}'...[/cyan]")
        responses = client.chat_stream(message_generator())
        for response in responses:
            console.print(f"[green]Relay response from '{response.sender}':[/green] {response.text}")
    except Exception as e:
        console.print(f"[red]Chat error:[/red] {e}")
    finally:
        client.close()

def share_file(file_path: str, target: str):
    import os
    if not os.path.exists(file_path):
        console.print(f"[red]ERROR:[/red] File {file_path} not found.")
        return

    client = CloakGrpcClient()
    filename = os.path.basename(file_path)
    file_id = f"file_{int(time.time())}"
    
    def chunk_generator():
        with open(file_path, "rb") as f:
            chunk_index = 0
            while True:
                data = f.read(1024 * 64) # 64KB chunks
                is_last = len(data) < (1024 * 64)
                yield cloakmesh_pb2.FileChunk(
                    file_id=file_id,
                    filename=filename,
                    data=data,
                    chunk_index=chunk_index,
                    is_last=is_last or not data
                )
                if is_last or not data:
                    break
                chunk_index += 1

    try:
        console.print(f"[cyan]Sharing file '{filename}' with {target}...[/cyan]")
        ack = client.file_transfer(chunk_generator())
        if ack and ack.success:
            console.print(f"[green]SUCCESS:[/green] {ack.message}")
        else:
            console.print(f"[red]FAILED:[/red] {ack.message if ack else 'Unknown error'}")
    except Exception as e:
        console.print(f"[red]Transfer error:[/red] {e}")
    finally:
        client.close()

def listen_chat():
    client = CloakGrpcClient()
    console.print("[cyan]Listening for incoming chat messages... Press Ctrl+C to stop.[/cyan]")
    
    def message_generator():
        # Send an initial empty message to keep the stream open
        ts = Timestamp()
        ts.FromSeconds(int(time.time()))
        yield cloakmesh_pb2.ChatMessage(sender="listener", text="", sent_at=ts)
        while True:
            time.sleep(1)
            
    try:
        responses = client.chat_stream(message_generator())
        for response in responses:
            if response.text:
                console.print(f"[green]{response.sender}:[/green] {response.text}")
    except KeyboardInterrupt:
        console.print("\n[yellow]Stopped listening.[/yellow]")
    except Exception as e:
        console.print(f"[red]Chat error:[/red] {e}")
    finally:
        client.close()

def receive_file():
    # In a full implementation, this would connect to a ReceiveFileStream RPC.
    # For Phase 2/3, we simulate receiving a file.
    console.print("[cyan]Waiting for incoming files...[/cyan]")
    time.sleep(2)
    console.print("[green]Incoming file:[/green] secret.txt (35 bytes)")
    console.print("[cyan]Saved to:[/cyan] ./downloads/secret.txt")

def view_chat_history():
    from rich.table import Table
    table = Table(title="Mesh Message History")
    table.add_column("Timestamp", style="dim")
    table.add_column("Sender", style="cyan")
    table.add_column("Message", style="green")
    
    # Mock data representing persistent mailbox retrieval
    table.add_row("2026-05-21 14:22", "Stanlley", "Hello through the onion!")
    table.add_row("2026-05-21 14:30", "Relay-1", "Awaiting handshake...")
    console.print(table)

def list_received_files():
    from rich.table import Table
    table = Table(title="Received Mesh Files")
    table.add_column("Filename", style="cyan")
    table.add_column("Size", style="magenta")
    table.add_column("Origin", style="dim")
    table.add_column("Status", style="bold green")
    
    table.add_row("secret.txt", "35B", "ahqw6...cloak", "DECRYPTED")
    table.add_row("schema.pdf", "1.2MB", "node-7...cloak", "READY")
    console.print(table)

