<div style="text-align:center">
    <h1> Cloudlet</h1>
    <p>The almost fast FaaS</p>
    <img src="./assets/demo.gif" alt="Demo" />
</div>

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Prerequisites](#prerequisites)
- [Run Locally](#run-locally)
  - [Clone the project](#clone-the-project)
  - [Setup](#setup)
  - [Start the VMM](#start-the-vmm)
  - [Run the API](#run-the-api)
  - [Send the request using the cli](#send-the-request-using-the-cli)
- [Architecture](#architecture)

## Prerequisites

Basic dependencies (with apt):

```bash
apt install build-essential cmake pkg-config libssl-dev flex bison libelf-dev
```

Others:

- add rust musl target: `rustup target add x86_64-unknown-linux-musl`
- add protoc binary: see [`github.com/protocolbuffers/protobuf`](https://github.com/protocolbuffers/protobuf)
- [just](https://github.com/casey/just): `cargo install just`

## Run Locally

### Clone the project

```bash
git clone https://github.com/virt-do/cloudlet
```

### Setup

Go to the project directory:

```bash
cd cloudlet
```

Create a TOML config file or update the [existing one](./src/agent/examples/config.toml):

```bash
cat << EOF > src/agent/examples/config.toml
workload-name = "fibonacci"
language = "rust"
action = "prepare-and-run"

[server]
address = "localhost"
port = 50051

[build]
source-code-path = "$(readlink -f ./src/agent/examples/main.rs)"
release = true
EOF
```

Make sure to update the source-code-path to the path of the source code you want to run.
Use an absolute path.

### Start the VMM

> [!WARNING]
> Make sure to replace $CARGO_PATH with the path to your cargo binary
> 
> ```bash
> export CARGO_PATH=$(which cargo)
> ```

```bash
sudo -E capsh --keep=1 --user=$USER --inh=cap_net_admin --addamb=cap_net_admin -- -c  'RUST_BACKTRACE=1 '$CARGO_PATH' run --bin vmm -- grpc'
```

### Run the API

```bash
cargo run --bin api
```

### Send the request using the cli 

```bash
cargo run --bin cli run --config-path src/agent/examples/config.toml
```

> [!NOTE]
> If it's your first time running the request, `cloudlet` will have to compile a kernel and an initramfs image.
> This will take a while, so make sure you do something else while you wait...

## Architecture

Here is a simple sequence diagram of Cloudlet:

```mermaid
sequenceDiagram
    participant CLI
    participant API
    participant VMM
    participant Agent

    CLI->>API: HTTP Request /run
    API->>VMM: gRPC Request to create VM
    VMM->>Agent: Creation of the VM
    VMM->>Agent: gRPC Request to the agent
    Agent->>Agent: Build and run code
    Agent-->>VMM: Stream Response
    VMM-->>API: Stream Response
    API-->>CLI: HTTP Response
```

1. The CLI sends an HTTP request to the API which in turn sends a gRPC request to the VMM
2. The VMM then creates a VM
3. When a VM starts it boots on the agent which holds another gRPC server to handle requests
4. The agent then builds and runs the code
5. The response is streamed back to the VMM and then to the API and finally to the CLI.
