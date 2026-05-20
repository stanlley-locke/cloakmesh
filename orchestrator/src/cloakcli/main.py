import typer
from rich.console import Console

app = typer.Typer(name="cloakcli", help="CloakMesh node management CLI")
console = Console()

node_app = typer.Typer(help="Node lifecycle commands")
dht_app = typer.Typer(help="DHT operations")
auth_app = typer.Typer(help="Capability token management")
chat_app = typer.Typer(help="Mesh communication")
file_app = typer.Typer(help="Mesh file sharing")

app.add_typer(node_app, name="node")
app.add_typer(dht_app, name="dht")
app.add_typer(auth_app, name="auth")
app.add_typer(chat_app, name="chat")
app.add_typer(file_app, name="file")


@node_app.command("start")
def node_start(
    port: int = 4001,
    bootstrap: str = "",
    config: str = "configs/default.toml",
):
    """Start a CloakMesh core node."""
    from cloakcli.node_manager import NodeManager
    mgr = NodeManager(port=port, bootstrap=bootstrap, config=config)
    mgr.start()


@node_app.command("stop")
def node_stop(port: int = 4001):
    """Stop a running node."""
    from cloakcli.node_manager import NodeManager
    NodeManager(port=port).stop()


@node_app.command("status")
def node_status(
    host: str = "127.0.0.1",
    port: int = 4001,
):
    """Check node health."""
    from cloakcli.health_checker import check_health
    check_health(host, port)


@dht_app.command("publish")
def dht_publish(
    address: str,
    config: str = "configs/default.toml"
):
    """Publish a .cloak descriptor to the DHT."""
    from cloakcli.dht_seeder import publish_descriptor
    publish_descriptor(address, config)


@dht_app.command("fetch")
def dht_fetch(address: str):
    """Fetch a .cloak descriptor from the DHT."""
    from cloakcli.dht_seeder import fetch_descriptor
    fetch_descriptor(address)


@auth_app.command("issue")
def auth_issue(
    address: str,
    scope: str = "read",
    ttl: int = 3600,
):
    """Issue a capability token."""
    from cloakcli.auth_generator import issue_token
    token = issue_token(address, scope, ttl)
    console.print(f"[green]Token:[/green] {token}")

@chat_app.command("send")
def chat_send(
    message: str,
    sender: str = "user",
):
    """Send a message through the mesh."""
    from cloakcli.communication import send_chat_message
    send_chat_message(message, sender)

@file_app.command("share")
def file_share(
    path: str,
    target: str,
):
    """Share a file with another peer."""
    from cloakcli.communication import share_file
    share_file(path, target)


if __name__ == "__main__":
    app()
