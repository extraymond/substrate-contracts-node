# test instruction

This repo was setup to explore solang's ability of executing compiled solidity code via pallet-contract. Target test environment is using runtime from substrate-contract-node and TestExternalities via the native-runtime, run it by:

```bash
SKIP_WASM_BUILD=1 cargo test -p contracts-node-runtime tests::contracts -- --nocapture
```
