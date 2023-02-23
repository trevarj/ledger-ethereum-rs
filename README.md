# Rust client for Ledger Ethereum App

This crate provides a basic Rust client library to communicate with the Ethereum App running on a Ledger Nano S/X device.

## Supported Instructions

- [ ] Get Public Key
- [ ] Sign Transaction
- [ ] Get App Configuration
- [ ] Sign Personal Message
- [ ] Provide Erc20 Token Information
- [ ] Sign Eip 712 Message
- [ ] Get Eth2 Public Key
- [ ] Set Eth2 Withdrawal Index
- [ ] Set External Plugin
- [ ] Provide Nft Information
- [ ] Set Plugin
- [ ] Perform Privacy Operation
- [ ] Eip712 Struct Def
- [ ] Eip712 Struct Impl
- [ ] Eip712 Filtering

## Testing

### Building app-ethereum

For debug printfs, edit Makefile and set DEBUG:=1 

```
git clone https://github.com/LedgerHQ/app-ethereum/
cd app-ethereum/
docker run --rm -ti -v "$(realpath .):/app" ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder-lite:latest
TARGET_NAME=TARGET_NANOS BOLOS_SDK=$NANOX_SDK make
mv build/app.elf speculos/apps/eth.elf
```

### Starting Speculos with docker

```
docker run --rm -it -v $(pwd)/apps:/speculos/apps --publish 41001:41001 speculos --display headless --model nanox --vnc-port 41001 apps/eth.elf --seed "6f0cd08f62d99e62ebb1e15f46df842c02380fd9f2abf987f0b5463adae25caeb564583bd413c9b7cbf0391808308332251e47696dd13688dc96b9edbccd981b"
```

Now you are ready to run the integration tests
