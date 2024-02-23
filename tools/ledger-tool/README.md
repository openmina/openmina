# Converts mina genesis ledger from json to binprot format suitable for OpenMina

Download json ledger [here](https://raw.githubusercontent.com/MinaProtocol/mina/2.0.0berkeley_rc1/genesis_ledgers/berkeley.json): 

Use the tool:

```
cargo run --release --bin ledger-tool -- --url https://raw.githubusercontent.com/MinaProtocol/mina/2.0.0berkeley_rc1/genesis_ledgers/berkeley.json --output genesis_ledgers/berkeley_genesis_ledger.bin
```
