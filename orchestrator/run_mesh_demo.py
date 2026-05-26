import os
import subprocess
import time

def run_cmd(cmd, env=None):
    print(f"\n[RUNNING] {cmd}")
    env_vars = os.environ.copy()
    if env:
        env_vars.update(env)
    return subprocess.run(cmd, shell=True, env=env_vars, text=True)

def main():
    print("=== CloakMesh Automated Integration Test ===")
    print("This script will seamlessly orchestrate a 2-node decentralized mesh network.")
    
    # 1. Reset Environment
    run_cmd("killall cloakmesh")
    run_cmd("rm -rf ../core/data_node-miner/sled_db")
    
    # 2. Start Node Alpha (The Host)
    public_alpha = "https://ideal-orbit-pjgjrvxgg7wjhw6-4001.app.github.dev"
    print("\n--- 1. STARTING NODE ALPHA (THE HOST) ---")
    run_cmd(f"poetry run cloakcli node-start --port 4001 --id node-alpha --public-addr {public_alpha}", env={"CLOAK_GRPC_PORT": "4001"})
    time.sleep(2)
    
    # 3. Host Site
    print("\n--- 2. HOSTING STATIC SITE ---")
    result = subprocess.run(f"poetry run cloakcli host-static my_darknet_site 8080 --intro-point {public_alpha}", shell=True, env={"CLOAK_GRPC_PORT": "4001"}, capture_output=True, text=True)
    print(result.stdout)
    
    address = None
    for line in result.stdout.split('\n'):
        if line.strip().endswith(".cloak"):
            address = line.strip().split()[-1]
            break
            
    if not address:
        print("[ERROR] Failed to parse .cloak address from host-static output.")
        return
        
    print(f"-> Successfully Hosted Address: {address}")
    
    # 4. Start Node Beta (The Client)
    public_beta = "https://ideal-orbit-pjgjrvxgg7wjhw6-4002.app.github.dev"
    print("\n--- 3. STARTING NODE BETA (THE CLIENT) ---")
    run_cmd(f"poetry run cloakcli node-start --port 4002 --id node-beta --bootstrap {public_alpha} --public-addr {public_beta}", env={"CLOAK_GRPC_PORT": "4002"})
    
    print("\n--- 4. WAITING FOR DHT REPLICATION (5s) ---")
    time.sleep(5)
    
    # 5. Fetch
    print(f"\n--- 5. FETCHING DHT DESCRIPTOR FROM NODE BETA ---")
    run_cmd(f"poetry run cloakcli dht-fetch {address}", env={"CLOAK_GRPC_PORT": "4002"})
    
    # 6. Browse
    print(f"\n--- 6. BROWSING SITE VIA SOCKS5 FROM NODE BETA ---")
    run_cmd(f"poetry run cloakcli browse {address}", env={"CLOAK_GRPC_PORT": "4002"})

if __name__ == "__main__":
    main()
