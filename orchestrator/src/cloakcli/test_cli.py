import typer
from rich.console import Console

app = typer.Typer(add_completion=False)
console = Console()

@app.command()
def issue(address: str, scope: str = "read", ttl: int = 3600):
    console.print(f"Addr: {address}, Scope: {scope}, TTL: {ttl}")

if __name__ == "__main__":
    app()
