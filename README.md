# Rust client for Ledger Ethereum App

This crate provides a basic Rust client library to communicate with the Ethereum App running on a Ledger Nano S/X device.

## Supported Instructions

- [x] Get Public Key
- [x] Sign Transaction
- [x] Get App Configuration
- [ ] Sign Personal Message
- [x] Provide Erc20 Token Information
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

https://github.com/LedgerHQ/ledger-app-builder 

```
git clone https://github.com/LedgerHQ/app-ethereum/
cd app-ethereum/
docker run --rm -ti -v "$(realpath .):/app" ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder-lite:latest
# inside docker env
make clean && make -j DEBUG=1 CHAIN=goerli NFT_TESTING_KEY=1 BOLOS_SDK=$NANOX_SDK && mv bin/app.elf tests/speculos/elfs/nanox_goerli.elf
```

### Starting Speculos

After installing app-ethereum requirements, you can run speculos with the following:

```
cd app-ethereum/tests/speculos
speculos --display headless ./elfs/nanox_goerli.elf --seed "6f0cd08f62d99e62ebb1e15f46df842c02380fd9f2abf987f0b5463adae25caeb564583bd413c9b7cbf0391808308332251e47696dd13688dc96b9edbccd981b"

```

Now you are ready to run the integration tests
