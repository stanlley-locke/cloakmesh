import subprocess
import signal
import os
import json
from pathlib import Path
from typing import List, Optional
from rich.console import Console

console = Console()


class NodeManager:
    """Manages CloakMesh node processes via subprocess and PID files.

    The binary is looked up in order:
      1. ./target/release/cloakmesh  (production build)
      2. ./target/debug/cloakmesh    (debug build)
    """

    PID_DIR = Path("/tmp/cloakmesh")
    LOG_DIR = Path("/tmp/cloakmesh/logs")

    def __init__(
        self,
        port: int = 4001,
        node_id: str = "",
        bootstrap: str = "",
        bootstrap_peers: Optional[List[str]] = None,
        peers_file: Optional[str] = None,
        public_addr: Optional[str] = None,
        mine_atk: bool = False,
        config: str = "configs/default.toml",
    ):
        self.port = port
        self.node_id = node_id
        self.bootstrap = bootstrap                  # single legacy peer
        self.bootstrap_peers = bootstrap_peers or []  # multi-peer list
        self.peers_file = peers_file
        self.public_addr = public_addr
        self.mine_atk = mine_atk
        self.config = config
        self.pid_file = self.PID_DIR / f"node-{port}.pid"
        self.log_file = self.LOG_DIR / f"node-{port}.log"

    def _find_binary(self) -> Path:
        """Return the path to the cloakmesh binary (release preferred)."""
        for candidate in [
            Path("./target/release/cloakmesh"),
            Path("./target/debug/cloakmesh"),
            Path("../target/release/cloakmesh"),
            Path("../target/debug/cloakmesh"),
        ]:
            if candidate.exists():
                return candidate
        raise FileNotFoundError(
            "cloakmesh binary not found. Run 'cargo build --release' in the core/ directory."
        )

    def build_command(self) -> List[str]:
        """Build the full CLI command for the node subprocess."""
        binary = self._find_binary()
        cmd = [str(binary), "--port", str(self.port), "--config", self.config]

        if self.node_id:
            cmd += ["--id", self.node_id]

        # Bootstrap peers: legacy single string
        if self.bootstrap:
            cmd += ["--bootstrap", self.bootstrap]

        # Bootstrap peers: multi-peer list
        for peer in self.bootstrap_peers:
            if peer:
                cmd += ["--bootstrap", peer]

        # JSON peers file
        if self.peers_file:
            cmd += ["--peers-file", self.peers_file]

        # Public address announcement
        if self.public_addr:
            cmd += ["--public-addr", self.public_addr]

        if self.mine_atk:
            cmd += ["--mine-atk"]

        return cmd

    def start(self) -> None:
        """Start the node as a background process."""
        self.PID_DIR.mkdir(parents=True, exist_ok=True)
        self.LOG_DIR.mkdir(parents=True, exist_ok=True)

        if self.pid_file.exists():
            pid = int(self.pid_file.read_text())
            try:
                os.kill(pid, 0)  # Check if process exists
                console.print(
                    f"[yellow]Node on port {self.port} is already running (pid {pid})[/yellow]"
                )
                return
            except ProcessLookupError:
                self.pid_file.unlink(missing_ok=True)

        cmd = self.build_command()
        console.print(f"[dim]Starting:[/dim] {' '.join(cmd)}")

        log_handle = open(self.log_file, "a")
        proc = subprocess.Popen(
            cmd,
            stdout=log_handle,
            stderr=log_handle,
        )

        self.pid_file.write_text(str(proc.pid))
        socks5_port = self.port + 5049
        console.print(
            f"[green]Node started[/green] on gRPC port [cyan]{self.port}[/cyan] "
            f"| SOCKS5 [cyan]{socks5_port}[/cyan] "
            f"| pid [dim]{proc.pid}[/dim] "
            f"| log [dim]{self.log_file}[/dim]"
        )

    def stop(self) -> None:
        """Stop a running node by sending SIGTERM."""
        if not self.pid_file.exists():
            console.print(f"[yellow]No node running on port {self.port}[/yellow]")
            return
        pid = int(self.pid_file.read_text())
        try:
            os.kill(pid, signal.SIGTERM)
            self.pid_file.unlink()
            console.print(f"[green]Node {pid} stopped[/green]")
        except ProcessLookupError:
            console.print(f"[red]Process {pid} not found[/red]")
            self.pid_file.unlink(missing_ok=True)

    def status(self) -> dict:
        """Return a status dict: {running, pid, port, log_file}."""
        if not self.pid_file.exists():
            return {"running": False, "pid": None, "port": self.port}
        pid = int(self.pid_file.read_text())
        try:
            os.kill(pid, 0)
            return {"running": True, "pid": pid, "port": self.port, "log": str(self.log_file)}
        except ProcessLookupError:
            self.pid_file.unlink(missing_ok=True)
            return {"running": False, "pid": pid, "port": self.port}

    @classmethod
    def list_running(cls) -> List[dict]:
        """List all running nodes based on PID files in PID_DIR."""
        cls.PID_DIR.mkdir(parents=True, exist_ok=True)
        results = []
        for pid_file in sorted(cls.PID_DIR.glob("node-*.pid")):
            try:
                port = int(pid_file.stem.split("-")[1])
                pid = int(pid_file.read_text())
                try:
                    os.kill(pid, 0)
                    results.append({"running": True, "pid": pid, "port": port})
                except ProcessLookupError:
                    pid_file.unlink(missing_ok=True)
                    results.append({"running": False, "pid": pid, "port": port, "stale": True})
            except (ValueError, IndexError):
                continue
        return results

    @classmethod
    def generate_peers_file(cls, peers: List[str], output_path: str, network: str = "cloakmesh-devnet") -> None:
        """Generate a JSON peers file from a list of address strings."""
        data = {
            "_format": "v1",
            "network": network,
            "peers": [
                {"address": addr}
                for addr in peers
                if addr
            ]
        }
        Path(output_path).write_text(json.dumps(data, indent=2))
        console.print(f"[green]Peers file written:[/green] {output_path} ({len(data['peers'])} peers)")
