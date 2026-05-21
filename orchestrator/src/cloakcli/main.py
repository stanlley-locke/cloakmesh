import click
from rich.console import Console
from rich.table import Table

console = Console()

@click.group(help="CloakMesh node management CLI")
def cli():
    pass

@cli.group(help="Node management")
def node():
    pass

@node.command(name="status")
@click.option("--host", default="127.0.0.1", help="Node host")
@click.option("--port", default=4001, help="Node port")
def node_status(host, port):
    from cloakcli.health_checker import check_health
    check_health(host, port)

@node.command(name="host")
@click.argument("address")
@click.argument("local_port", type=int)
def node_host(address, local_port):
    from cloakcli.api.grpc_client import CloakGrpcClient
    client = CloakGrpcClient()
    response = client.host_site(address, local_port)
    if response and response.success:
        console.print(f"[green]SUCCESS:[/green] {response.message}")
    else:
        console.print(f"[red]FAILED:[/red] {response.message if response else 'Unknown error'}")
    client.close()

@node.command(name="circuits")
def node_circuits():
    table = Table(title="Active Onion Circuits")
    table.add_column("Circuit ID")
    table.add_column("Hops")
    table.add_column("Status")
    table.add_row("e796bf4a...", "3", "READY")
    console.print(table)

@cli.group(help="DHT operations")
def dht():
    pass

@dht.command(name="publish")
@click.argument("address")
@click.option("--config", default="configs/default.toml", help="Config")
def dht_publish(address, config):
    from cloakcli.dht_seeder import publish_descriptor
    publish_descriptor(address, config)

@dht.command(name="fetch")
@click.argument("address")
def dht_fetch(address):
    from cloakcli.dht_seeder import fetch_descriptor
    fetch_descriptor(address)

@cli.group(help="Auth management")
def auth():
    pass

@auth.command(name="issue")
@click.argument("address")
@click.option("--scope", default="read", help="Scope")
@click.option("--ttl", default=3600, type=int, help="TTL")
def auth_issue(address, scope, ttl):
    from cloakcli.auth_generator import issue_token
    token = issue_token(address, scope, ttl)
    console.print(f"Token: {token}")

@cli.group(help="Communication")
def chat():
    pass

@chat.command(name="send")
@click.argument("message")
@click.option("--sender", default="user", help="Sender")
def chat_send(message, sender):
    from cloakcli.communication import send_chat_message
    send_chat_message(message, sender)

@chat.command(name="listen")
def chat_listen():
    from cloakcli.communication import listen_chat
    listen_chat()

@chat.command(name="history")
def chat_history():
    """View the log of received mesh messages."""
    from cloakcli.communication import view_chat_history
    view_chat_history()

@cli.group(help="File transfer")
def file():
    pass

@file.command(name="list")
def file_list():
    """List securely received files."""
    from cloakcli.communication import list_received_files
    list_received_files()

@file.command(name="share")
@click.argument("path")
@click.argument("target")
def file_share(path, target):
    from cloakcli.communication import share_file
    share_file(path, target)

@file.command(name="receive")
def file_receive():
    from cloakcli.communication import receive_file
    receive_file()

if __name__ == "__main__":
    cli()
