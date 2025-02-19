# BlockTalk

> ⚠️ WARNING: This library is pre-alpha and under active development. APIs may change significantly between versions. Not recommended for production use.

A Rust library for interacting with Bitcoin nodes via IPC.

## Overview
BlockTalk provides a high-level API to connect to `bitcoin-node` process, subscribe to blockchain events, and query information about the blockchain state.

### Development status
- [x] Connect / Disconnect
- [x] Query blockchain data 
- [ ] Subscribe to real-time blockchain events

## License
MIT License

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements 
This project is heavily inspired by Antoine Poinsot's [prototype](https://github.com/darosior/core_bdk_wallet) which is based on @ryanofsky's work on [Multiprocess Bitcoin Design Document](https://github.com/ryanofsky/bitcoin/blob/pr/ipc/doc/design/multiprocess.md)