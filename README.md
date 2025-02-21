# BlockTalk

> ⚠️ WARNING: This library is pre-alpha and under active development. APIs may change significantly between versions. Not recommended for production use.

A Rust library for interacting with Bitcoin nodes via IPC.

## Overview
BlockTalk provides a high-level API to connect to `bitcoin-node` process, subscribe to blockchain events, and query information about the blockchain state.

## Development status
- [x] Connect / Disconnect
- [x] Query blockchain data 
- [ ] Subscribe to real-time blockchain events

## Setup Guide
### Build Bitcoin Core with Multiprocess Support
First, build Bitcoin Core with multiprocess support enabled using [PR#29409](https://github.com/bitcoin/bitcoin/pull/29409).

```bash
# Clone Bitcoin Core and checkout the PR
git clone https://github.com/bitcoin/bitcoin.git
cd bitcoin
git fetch origin pull/29409/head:pr29409
git checkout pr29409

# Build dependencies with multiprocess support
make -C depends HOST=aarch64-apple-darwin MULTIPROCESS=1 NO_QT=1

# Configure and build Bitcoin Core
export HOST_PLATFORM="aarch64-apple-darwin"
cmake -B multiprocbuild/ --toolchain=depends/$HOST_PLATFORM/toolchain.cmake
cmake --build multiprocbuild/ --parallel $(sysctl -n hw.logicalcpu)
```
For more details on multiprocess Bitcoin, refer to the [documentation](https://github.com/bitcoin/bitcoin/blob/master/doc/multiprocess.md#installation).

### Set Up and Run Bitcoin Node
Create a directory for the node and start the node in regtest mode:

```bash
# Create data directory
mkdir -p datadir_blocktalk

# Start Bitcoin node
./multiprocbuild/src/bitcoin-node \
    -regtest \
    -datadir=$PWD/datadir_blocktalk \
    -server=0 \
    -port=19444 \
    -connect=127.0.0.1:18444 \
    -ipcbind=unix \
    -debug=ipc
```

#### Node Configuration Parameters
- `-regtest`: Use regression test mode (local testing chain)
- `-server=0`: Disable RPC server as we'll use IPC
- `-ipcbind=unix`: Enable Unix domain socket for IPC
- `-debug=ipc`: Enable IPC debugging logs

### Run BlockTalk

```bash
# Clone BlockTalk repository
git clone https://github.com/yourusername/blocktalk.git
cd blocktalk

# Run the chain query example
cargo run --example chain_query
```

The examples expect Bitcoin Core and BlockTalk to be in sibling directories. If you have a different setup, update the `socket_path` in `examples/chain_query.rs`:

## License
MIT License

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements 
This project is heavily inspired by [@darosior](https://github.com/darosior)'s [prototype](https://github.com/darosior/core_bdk_wallet) which is based on [@ryanofsky](https://github.com/ryanofsky)'s work on [Multiprocess Bitcoin](https://github.com/ryanofsky/bitcoin/blob/pr/ipc/doc/design/multiprocess.md)