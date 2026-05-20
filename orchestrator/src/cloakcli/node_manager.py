import subprocess
import signal
import os
from pathlib import Path
from rich.console import Console

console = Console()


class NodeManager:
    PID_DIR = Path("/tmp/cloakmesh")

    def __init__(self, port: int = 4001, bootstrap: str = "", config: str = "configs/default.toml"):
        self.port = port
        self.bootstrap = bootstrap
        self.config = config
        self.pid_file = self.PID_DIR / f"node-{port}.pid"

    def start(self) -> None:
        self.PID_DIR.mkdir(exist_ok=True)
        cmd = ["./target/release/cloakmesh", "--port", str(self.port), "--config", self.config]
        if self.bootstrap:
            cmd += ["--bootstrap", self.bootstrap]
        proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        self.pid_file.write_text(str(proc.pid))
        console.print(f"[green]Node started[/green] on port {self.port} (pid {proc.pid})")

    def stop(self) -> None:
        if not self.pid_file.exists():
            console.print(f"[yellow]No PID file for port {self.port}[/yellow]")
            return
        pid = int(self.pid_file.read_text())
        try:
            os.kill(pid, signal.SIGTERM)
            self.pid_file.unlink()
            console.print(f"[green]Node {pid} stopped[/green]")
        except ProcessLookupError:
            console.print(f"[red]Process {pid} not found[/red]")
            self.pid_file.unlink(missing_ok=True)
