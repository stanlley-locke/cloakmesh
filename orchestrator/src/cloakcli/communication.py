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
