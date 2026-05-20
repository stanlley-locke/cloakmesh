import typer
from rich.console import Console

app = typer.Typer(name="cloakcli", help="CloakMesh node management CLI")
console = Console()

node_app = typer.Typer(help="Node lifecycle commands")
dht_app = typer.Typer(help="DHT operations")
auth_app = typer.Typer(help="Capability token management")

app.add_typer(node_app, name="node")
app.add_typer(dht_app, name="dht")
app.add_typer(auth_app, name="auth")


@node_app.command("start")
def node_start(
    port: int = typer.Option(4001, help="Listen port"),
    bootstrap: str = typer.Option("", help="Bootstrap peer address"),
    config: str = typer.Option("configs/default.toml", help="Config file path"),
) -> None:
    """Start a CloakMesh core node."""
    from cloakcli.node_manager import NodeManager
    mgr = NodeManager(port=port, bootstrap=bootstrap, config=config)
    mgr.start()


@node_app.command("stop")
def node_stop(port: int = typer.Option(4001)) -> None:
    """Stop a running node."""
    from cloakcli.node_manager import NodeManager
    NodeManager(port=port).stop()


@node_app.command("status")
def node_status(
    host: str = typer.Option("127.0.0.1", help="Node host"),
    port: int = typer.Option(4001, help="Node gRPC port"),
) -> None:
    """Check node health."""
    from cloakcli.health_checker import check_health
    check_health(host, port)


@dht_app.command("publish")
def dht_publish(address: str, config: str = "configs/default.toml") -> None:
    """Publish a .cloak descriptor to the DHT."""
    from cloakcli.dht_seeder import publish_descriptor
    publish_descriptor(address, config)


@dht_app.command("fetch")
def dht_fetch(address: str) -> None:
    """Fetch a .cloak descriptor from the DHT."""
    from cloakcli.dht_seeder import fetch_descriptor
    fetch_descriptor(address)


@auth_app.command("issue")
def auth_issue(
    address: str,
    scope: str = "read",
    ttl: int = typer.Option(3600, help="TTL in seconds"),
) -> None:
    """Issue a capability token."""
    from cloakcli.auth_generator import issue_token
    token = issue_token(address, scope, ttl)
    console.print(f"[green]Token:[/green] {token}")


if __name__ == "__main__":
    app()
