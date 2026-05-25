import time
import sys
import os
import threading
from typing import Optional
import typer
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.layout import Layout
from rich.live import Live
from rich import box
from rich.prompt import Prompt, Confirm, IntPrompt
from .wallet import wallet_app
from .storage import storage_app

console = Console()

# The main CLI entry point
app = typer.Typer(
    help="CloakMesh Orchestrator CLI. Manage nodes, monitor circuits, and communicate over the darknet.",
    no_args_is_help=True
)

# Register sub-commands
app.add_typer(wallet_app, name="wallet")
app.add_typer(storage_app, name="storage")

# ── Dependency Helpers ────────────────────────────────────────────────────────

def get_client():
    from cloakcli.api.grpc_client import CloakGrpcClient
    return CloakGrpcClient()

def get_node_metrics():
    client = get_client()
    try:
        metrics = client.get_metrics()
        return metrics
    except Exception as e:
        return None
    finally:
        client.close()

# ── Live Dashboard ────────────────────────────────────────────────────────────

def generate_dashboard() -> Layout:
    layout = Layout()
    layout.split_column(
        Layout(name="header", size=3),
        Layout(name="main"),
        Layout(name="footer", size=3)
    )
    layout["main"].split_row(
        Layout(name="left_col", ratio=1),
        Layout(name="right_col", ratio=2)
    )
    layout["left_col"].split_column(
        Layout(name="metrics", ratio=1),
        Layout(name="circuits", ratio=1)
    )
    
    # Fetch Data
    metrics = get_node_metrics()
    
    # Header
    header_text = "[bold cyan]CloakMesh Node Commander[/bold cyan] - Live Telemetry"
    if not metrics:
        header_text += " [bold red](OFFLINE)[/bold red]"
    layout["header"].update(Panel(header_text, style="white on dark_blue"))
    
    # Metrics Panel
    metrics_table = Table(box=box.SIMPLE, show_header=False)
    metrics_table.add_column("Key", style="cyan")
    metrics_table.add_column("Value", style="magenta")
    
    if metrics:
        metrics_table.add_row("Address", metrics.cloak_address[:20] + "...")
        metrics_table.add_row("Uptime (s)", str(metrics.uptime_seconds))
        metrics_table.add_row("Active Circuits", str(metrics.active_circuits))
        metrics_table.add_row("DHT Entries", str(metrics.dht_entries))
        metrics_table.add_row("Bytes Relayed", f"{metrics.bytes_relayed / 1024 / 1024:.2f} MB")
        metrics_table.add_row("Avg Latency", f"{metrics.avg_circuit_latency_ms} ms")
        metrics_table.add_row("Reputation", f"{metrics.reputation_score:.3f}")
    else:
        metrics_table.add_row("Status", "Cannot reach Core Node")
        
    layout["metrics"].update(Panel(metrics_table, title="[bold]Core Metrics[/bold]", border_style="cyan"))
    
    # Circuits Panel
    circuits_table = Table(box=box.ROUNDED)
    circuits_table.add_column("Circuit ID", style="dim")
    circuits_table.add_column("Hops")
    circuits_table.add_column("Status")
    
    client = get_client()
    try:
        c_res = client.get_circuits()
        if c_res and hasattr(c_res, 'circuits'):
            for c in c_res.circuits[:5]: # Show top 5
                circuits_table.add_row(c.id[:8], str(len(c.hops)), c.status)
        else:
            circuits_table.add_row("None", "-", "-")
    except Exception:
        circuits_table.add_row("Error", "-", "-")
    finally:
        client.close()
        
    layout["circuits"].update(Panel(circuits_table, title="[bold]Active Circuits[/bold]", border_style="green"))
    
    # Right Column (Logs / Activity)
    import os
    logs = "No logs found."
    try:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        root_dir = os.path.abspath(os.path.join(script_dir, "..", "..", "..", ".."))
        core_log_path = os.path.join(root_dir, "core.log")
        with open(core_log_path, "r") as f:
            lines = f.readlines()[-20:]
            logs = "".join(lines)
    except:
        pass
    
    layout["right_col"].update(Panel(logs, title="[bold]Core Event Stream (JSON)[/bold]", border_style="yellow"))
    
    # Footer
    layout["footer"].update(Panel("Press Ctrl+C to exit dashboard", style="dim"))
    
    return layout

@app.command(help="Launch the live terminal dashboard")
def dash():
    """Starts the real-time CloakMesh node monitoring dashboard."""
    with Live(generate_dashboard(), refresh_per_second=1, screen=True):
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            pass
    console.print("[green]Dashboard exited.[/green]")

# ── Interactive Menu ──────────────────────────────────────────────────────────

@app.command(help="Launch the interactive CLI menu")
def interactive():
    """Starts an interactive prompt loop for managing the node."""
    console.clear()
    console.print(Panel.fit("[bold cyan]Welcome to the CloakMesh Interactive Console[/bold cyan]\nType 'help' to see commands.", border_style="cyan"))
    
    while True:
        try:
            cmd = Prompt.ask("\n[bold cyan]cloak[/bold cyan]>[white]").strip().lower()
            
            if cmd in ("exit", "quit", "q"):
                break
            elif cmd == "help":
                table = Table(box=box.SIMPLE, show_header=False)
                table.add_column("Command", style="cyan")
                table.add_column("Description")
                table.add_row("dash", "Open live dashboard")
                table.add_row("status", "View current node metrics")
                table.add_row("ping", "Ping the node (keepalive check)")
                table.add_row("circuits", "View active routing circuits")
                table.add_row("relays", "View known DHT relay peers")
                table.add_row("stream-metrics", "Stream live telemetry (Ctrl+C to stop)")
                table.add_row("dht-fetch", "Fetch an address from the DHT")
                table.add_row("dht-publish", "Publish an address to the DHT")
                table.add_row("site-init", "Initialize a beautiful static .cloak website")
                table.add_row("host-static", "Host a static website directory on the mesh")
                table.add_row("host", "Expose an existing local port to the mesh")
                table.add_row("browse", "Browse a .cloak site via SOCKS5 proxy")
                table.add_row("auth", "Issue a capability token")
                table.add_row("chat", "Send an encrypted chat message")
                table.add_row("listen", "Listen for incoming chat messages")
                table.add_row("history", "View chat message history")
                table.add_row("file-send", "Share a file over the mesh")
                table.add_row("file-recv", "Listen for incoming files")
                table.add_row("list-files", "List received files")
                table.add_row("wallet", "Manage ATK balance and transfers")
                table.add_row("storage", "Upload/Download decentralized files")
                table.add_row("address", "View your node's .cloak address")
                table.add_row("node-start", "Start a background node process")
                table.add_row("node-stop", "Stop a background node process")
                table.add_row("clear", "Clear screen")
                console.print(table)
                
            elif cmd == "ping":
                ping()
            elif cmd == "relays":
                relays()
            elif cmd == "stream-metrics":
                stream_metrics()
            elif cmd == "status":
                status()
            elif cmd == "dash":
                dash()
            elif cmd == "circuits":
                circuits()
            elif cmd == "history":
                view_chat_history()
            elif cmd == "list-files":
                list_files()
            elif cmd == "dht-fetch":
                target = Prompt.ask("Enter address to lookup")
                dht_fetch(target)
            elif cmd == "dht-publish":
                address = Prompt.ask("Enter .cloak address to publish")
                dht_publish(address)
            elif cmd == "site-init":
                dir_name = Prompt.ask("Enter directory name for the site", default="./mysite")
                site_init(dir_name)
            elif cmd == "host-static":
                dir_name = Prompt.ask("Enter static site directory")
                port = IntPrompt.ask("Enter local port to run the server on", default=8080)
                host_static(dir_name, port)
            elif cmd == "host":
                addr = Prompt.ask("Enter .cloak address")
                port = IntPrompt.ask("Enter local port")
                host(addr, port)
            elif cmd == "browse":
                addr = Prompt.ask("Enter .cloak address to fetch")
                browse(addr)
            elif cmd == "auth":
                addr = Prompt.ask("Enter target .cloak address")
                scope = Prompt.ask("Enter scope", default="read")
                ttl = IntPrompt.ask("Enter TTL in seconds", default=3600)
                auth_issue(addr, scope, ttl)
            elif cmd == "chat":
                target = Prompt.ask("Enter destination address")
                msg = Prompt.ask("Enter message")
                chat_send(msg, target)
            elif cmd == "listen":
                listen_chat()
            elif cmd == "file-send":
                path = Prompt.ask("Enter file path to send")
                target = Prompt.ask("Enter destination address")
                file_send(path, target)
            elif cmd == "file-recv":
                file_recv()
            elif cmd.startswith("wallet "):
                parts = cmd.split()
                if len(parts) > 1 and parts[1] == "balance":
                    from .wallet import balance
                    balance()
                elif len(parts) > 3 and parts[1] == "transfer":
                    from .wallet import transfer
                    transfer(float(parts[2]), parts[3])
                else:
                    console.print("Usage: wallet balance | wallet transfer <amount> <address>")
            elif cmd.startswith("storage "):
                parts = cmd.split()
                if len(parts) > 2 and parts[1] == "upload":
                    from .storage import upload
                    upload(parts[2])
                elif len(parts) > 2 and parts[1] == "download":
                    from .storage import download
                    download(parts[2])
                else:
                    console.print("Usage: storage upload <file> | storage download <hash>")
            elif cmd == "address":
                address_cmd()
            elif cmd == "clear":
                console.clear()
            elif cmd:
                console.print(f"[red]Unknown command:[/red] {cmd}")
        except KeyboardInterrupt:
            break
        except Exception as e:
            console.print(f"[red]Error:[/red] {e}")

# ── Individual Commands ───────────────────────────────────────────────────────

@app.command(help="Print current node metrics")
def status():
    metrics = get_node_metrics()
    if not metrics:
        console.print("[red]Node is unreachable.[/red]")
        return
        
    table = Table(title="Node Status", box=box.MINIMAL_DOUBLE_HEAD)
    table.add_column("Metric", style="cyan")
    table.add_column("Value", style="white")
    table.add_row("Address", metrics.cloak_address)
    table.add_row("Active Circuits", str(metrics.active_circuits))
    table.add_row("Bytes Relayed", str(metrics.bytes_relayed))
    table.add_row("Reputation", str(metrics.reputation_score))
    console.print(table)

@app.command(help="List active onion circuits")
def circuits():
    client = get_client()
    try:
        res = client.get_circuits()
        if res and hasattr(res, 'circuits'):
            table = Table(title="Active Onion Circuits", box=box.ROUNDED)
            table.add_column("ID", style="dim")
            table.add_column("Hops")
            table.add_column("Status", style="green")
            table.add_column("Latency")
            for c in res.circuits:
                table.add_row(c.id, str(len(c.hops)), c.status, f"{c.latency_ms}ms")
            console.print(table)
        else:
            console.print("[yellow]No active circuits found.[/yellow]")
    except Exception as e:
        console.print(f"[red]Error fetching circuits:[/red] {e}")
    finally:
        client.close()

@app.command(help="Host a local service on the mesh")
def host(address: str, port: int = typer.Argument(8080)):
    client = get_client()
    try:
        response = client.host_site(address, port)
        if response and response.success:
            console.print(f"[green]SUCCESS:[/green] Service bridged to {address}")
        else:
            console.print(f"[red]FAILED:[/red] {getattr(response, 'message', 'Unknown Error')}")
    finally:
        client.close()

@app.command(help="Browse a .cloak site via local SOCKS5 proxy")
def browse(address: str):
    address = address.replace('\n', '').replace('\r', '').strip()
    import socket
    port = int(os.environ.get("CLOAK_GRPC_PORT", 4001))
    proxy_port = port + 5049
    console.print(f"[cyan]Attempting to fetch index from {address} via Core SOCKS5 Proxy (127.0.0.1:{proxy_port})...[/cyan]")
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(5)
        s.connect(("127.0.0.1", proxy_port))
        s.sendall(b'\x05\x01\x00')
        resp = s.recv(2)
        if resp != b'\x05\x00':
            raise Exception("SOCKS5 auth failed")
        addr_bytes = address.encode('utf-8')
        s.sendall(b'\x05\x01\x00\x03' + bytes([len(addr_bytes)]) + addr_bytes + b'\x00\x50')
        socks_resp = s.recv(10)
        if len(socks_resp) < 2 or socks_resp[1] != 0x00:
            raise Exception("SOCKS5 connection refused by destination")
        
        s.sendall(f"GET / HTTP/1.0\r\nHost: {address}\r\n\r\n".encode())
        raw = b""
        while True:
            chunk = s.recv(4096)
            if not chunk:
                break
            raw += chunk
            if len(raw) > 1024 * 50:
                break
        
        console.print("\n[green]--- RESPONSE ---[/green]")
        console.print(raw.decode('utf-8', errors='ignore'))
        console.print("[green]----------------[/green]")
    except Exception as e:
        console.print(f"[red]Browsing Error:[/red] {e}")

@app.command(help="Initialize a static .cloak website")
def site_init(dir_name: str):
    import os
    os.makedirs(dir_name, exist_ok=True)
    html_content = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CloakMesh Decentralized Site</title>
    <style>
        body { font-family: 'Inter', sans-serif; background-color: #0f172a; color: #f8fafc; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .card { background: rgba(30, 41, 59, 0.7); backdrop-filter: blur(10px); padding: 3rem; border-radius: 1rem; border: 1px solid #334155; text-align: center; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5); }
        h1 { color: #38bdf8; margin-bottom: 0.5rem; }
        p { color: #94a3b8; font-size: 1.1rem; }
        .badge { display: inline-block; padding: 0.25rem 0.75rem; background: #0ea5e9; color: #fff; border-radius: 9999px; font-size: 0.875rem; font-weight: bold; margin-top: 1rem; }
    </style>
</head>
<body>
    <div class="card">
        <h1>Welcome to the Deep Web</h1>
        <p>This site is hosted entirely over the CloakMesh P2P Onion Network.</p>
        <div class="badge">Decentralized</div>
    </div>
</body>
</html>"""
    with open(os.path.join(dir_name, "index.html"), "w") as f:
        f.write(html_content)
    console.print(f"[green]SUCCESS:[/green] Created beautiful static site in {dir_name}/index.html")

@app.command(help="Host a static site folder on the network")
def host_static(dir_name: str, port: int = typer.Argument(8080)):
    import os
    import subprocess
    import threading
    
    if not os.path.exists(dir_name):
        console.print(f"[red]Error:[/red] Directory {dir_name} not found.")
        return

    metrics = get_node_metrics()
    if not metrics:
        console.print("[red]Node is unreachable.[/red]")
        return
    address = metrics.cloak_address
        
    def run_server():
        subprocess.run(["python3", "-m", "http.server", str(port), "-d", dir_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
    t = threading.Thread(target=run_server, daemon=True)
    t.start()
    
    client = get_client()
    try:
        response = client.host_site(address, port)
        if response and response.success:
            console.print(f"[green]SUCCESS:[/green] Started static server on port {port} and bridged to [bold cyan]{address}[/bold cyan]")
            from cloakcli.dht_seeder import publish_descriptor
            console.print("[cyan]Auto-publishing to DHT...[/cyan]")
            publish_descriptor(address, "configs/default.toml")
            console.print(f"[magenta]Your site is LIVE in the background![/magenta] Browse it with: cloakcli browse {address}")
        else:
            console.print(f"[red]FAILED to bridge:[/red] {getattr(response, 'message', 'Unknown Error')}")
    finally:
        client.close()

@app.command(name="address", help="View your node's primary .cloak address")
def address_cmd():
    metrics = get_node_metrics()
    if not metrics:
        console.print("[red]Node is unreachable.[/red]")
        return
    
    console.print(Panel(
        f"[bold cyan]{metrics.cloak_address}[/bold cyan]",
        title="[bold green]Your Node Identity[/bold green]",
        expand=False,
        border_style="green"
    ))

@app.command(help="Fetch a DHT descriptor")
def dht_fetch(address: str):
    from cloakcli.dht_seeder import fetch_descriptor
    fetch_descriptor(address)

@app.command(help="Publish a DHT descriptor")
def dht_publish(address: str):
    from cloakcli.dht_seeder import publish_descriptor
    publish_descriptor(address, "configs/default.toml")

@app.command(help="Issue a capability token")
def auth_issue(address: str, scope: str = "read", ttl: int = 3600):
    from cloakcli.auth_generator import issue_token
    token = issue_token(address, scope, ttl)
    console.print(f"[green]Token Issued:[/green] {token}")

@app.command(help="Send a chat message")
def chat_send(message: str, target: str):
    from cloakcli.communication import send_chat_message
    send_chat_message(message, target)

@app.command(help="Listen for chat messages")
def listen_chat():
    from cloakcli.communication import listen_chat as listen
    listen()

@app.command(help="Send a file over the mesh")
def file_send(path: str, target: str):
    from cloakcli.communication import share_file
    share_file(path, target)

@app.command(help="Listen for incoming files")
def file_recv():
    from cloakcli.communication import receive_file
    receive_file()


@app.command(help="Ping the node — check keepalive and measure latency")
def ping():
    client = get_client()
    import time
    try:
        start = time.time()
        resp = client.ping()
        elapsed = (time.time() - start) * 1000
        if resp:
            console.print(f"[green]PONG[/green] node_id=[cyan]{resp.node_id[:16]}...[/cyan] latency=[yellow]{elapsed:.1f}ms[/yellow]")
        else:
            console.print("[red]No response — node may be offline.[/red]")
    except Exception as e:
        console.print(f"[red]Ping failed:[/red] {e}")
    finally:
        client.close()

@app.command(help="List known DHT relay peers")
def relays():
    client = get_client()
    try:
        res = client.get_relays()
        if res and hasattr(res, 'relays') and res.relays:
            table = Table(title="Known DHT Relay Peers", box=box.ROUNDED)
            table.add_column("Node ID", style="dim")
            table.add_column("Address", style="cyan")
            table.add_column("Reputation", style="magenta")
            table.add_column("Latency")
            for r in res.relays:
                table.add_row(
                    r.node_id[:16] + "...",
                    r.address,
                    str(getattr(r, 'reputation', 'N/A')),
                    f"{getattr(r, 'avg_latency_ms', 0)}ms"
                )
            console.print(table)
        else:
            console.print("[yellow]No relay peers known yet — bootstrap peers will appear here after connection.[/yellow]")
    except Exception as e:
        console.print(f"[red]Error fetching relays:[/red] {e}")
    finally:
        client.close()

@app.command(help="Stream live telemetry from the node (push every 2s). Press Ctrl+C to stop.")
def stream_metrics():
    client = get_client()
    console.print("[cyan]Streaming live metrics... Press Ctrl+C to stop.[/cyan]")
    try:
        from proto import telemetry_pb2
        port = int(os.getenv("CLOAK_GRPC_PORT", "4001"))
        import grpc
        from proto import telemetry_pb2_grpc
        channel = grpc.insecure_channel(f"127.0.0.1:{port}")
        stub = telemetry_pb2_grpc.TelemetryServiceStub(channel)
        req = telemetry_pb2.MetricsRequest(node_id="")
        for metric in stub.StreamMetrics(req):
            console.print(
                f"[dim]uptime={metric.uptime_seconds}s[/dim] "
                f"circuits=[cyan]{metric.active_circuits}[/cyan] "
                f"dht_entries=[magenta]{metric.dht_entries}[/magenta] "
                f"relayed=[green]{metric.bytes_relayed / 1024:.1f}KB[/green] "
                f"latency=[yellow]{metric.avg_circuit_latency_ms}ms[/yellow] "
                f"rep=[white]{metric.reputation_score:.3f}[/white]"
            )
    except KeyboardInterrupt:
        console.print("\n[yellow]Stopped streaming.[/yellow]")
    except Exception as e:
        console.print(f"[red]Stream error:[/red] {e}")
    finally:
        client.close()

@app.command(name="view-chat-history", help="View recent chat message history")
def view_chat_history():
    from cloakcli.communication import view_chat_history as _view
    _view()

@app.command(name="list-files", help="List received mesh files")
def list_files():
    from cloakcli.communication import list_received_files
    list_received_files()

@app.command(name="node-start", help="Start a CloakMesh node as a background process")
def node_start(
    port: int = typer.Option(4001, "--port", "-p", help="gRPC listen port"),
    node_id: str = typer.Option("", "--id", help="Node identifier"),
    bootstrap: str = typer.Option("", "--bootstrap", help="Bootstrap peer address (host:port)"),
    mine_atk: bool = typer.Option(False, "--mine-atk", help="Enable persistent storage and ATK mining"),
):
    from cloakcli.node_manager import NodeManager
    manager = NodeManager(port=port, bootstrap=bootstrap)
    manager.start()
    console.print(f"[green]Node started[/green] on gRPC port [cyan]{port}[/cyan] | SOCKS5 on [cyan]{port + 5049}[/cyan]")
    if node_id:
        console.print(f"[dim]Node ID: {node_id}[/dim]")

@app.command(name="node-stop", help="Stop a background CloakMesh node")
def node_stop(
    port: int = typer.Option(4001, "--port", "-p", help="gRPC port of the node to stop"),
):
    from cloakcli.node_manager import NodeManager
    manager = NodeManager(port=port)
    manager.stop()

@app.command(name="auth-verify", help="Verify a capability token JSON string")
def auth_verify(token_json: str):
    import json, time
    try:
        token = json.loads(token_json)
        now = int(time.time())
        expired = now > token.get("expires_at", 0)
        table = Table(title="Capability Token", box=box.ROUNDED)
        table.add_column("Field", style="cyan")
        table.add_column("Value")
        table.add_row("Token ID", token.get("token_id", "?"))
        table.add_row("Address", token.get("cloak_address", "?"))
        table.add_row("Scope", token.get("scope", "?"))
        table.add_row("Expires At", str(token.get("expires_at", "?")))
        table.add_row("Status", "[red]EXPIRED[/red]" if expired else "[green]VALID[/green]")
        console.print(table)
    except Exception as e:
        console.print(f"[red]Invalid token JSON:[/red] {e}")

if __name__ == "__main__":
    app()
