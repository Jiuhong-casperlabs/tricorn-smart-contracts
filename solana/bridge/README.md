# Tricorn Solana bridging contract

## Building the contract

Requirements:

0. [`rustup`](https://rustup.rs)
1. [Solana tools](https://docs.solana.com/cli/install-solana-cli-tools)
2. [Anchor](https://www.anchor-lang.com/docs/installation)

After this:

```
$ anchor build
```

The bridge program `.so` will be located at `target/bpfel-unknown-unknown/release/bridge.so`.

The generated Anchor IDL will be at `target/idl/bridge.json`. The IDL contains type and method definitions for the contract, and can be used for codegen using tools compatible with Anchor.

## Testing

By default, the tests are configured to run against a local Solana node. To quickly spin up a solana node, use:

```
$ anchor localnet
```

This will spin up a localnet with the bridge program already deployed.

To run tests, run:


```
$ cargo test
```

If you want to run tests against devnet, you can do so by setting the `SOLANA_CLUSTER` env variable, e.g.:

```
$ SOLANA_CLUSTER=devnet cargo test
```

By default this will use public Ankr RPC nodes.

## Deployment

To deploy the bridge program to devnet:

```
solana -u devnet program deploy target/bpfel-unknown-unknown/release/bridge.so --program-id test-keys/bridge-program.json
```

Most instructions require an initialized `Bridge` account to be created as well. This can be done by sending a `Initialize` instruction to the bridge program. The exact steps can be found in `init_bridge`@`programs-test/src/util/bridge.rs`. The specifics will depend on the language/SDK.

## Contract structure and remarks:

The contract is built with the help of the Anchor framework. This simplifies quite a few things when dealing with Solana.

The methods and logic are defined in `programs/bridge/src/lib.rs`. Entrypoints are commented where extra requirements cannot be expressed through Anchor.

Various other definitions are in `programs/bridge/src/definitions`. Most of the validation is using Anchor's built-in macro annotations over `Accounts` structs - refer to `programs/bridge/src/definitions/instructions.rs` for details.

Several methods in the contract require a signature from an offchain source to be provided to verify that the bridge backend authorized an operation. These signatures are verified using the Solana built-in SigVerify precompiles. Each signature requires a `nonce` value, which is 'consumed' after signature verification by creating a 0-size account at a PDA with the nonce as one of the seeds, making nonces non-reusable.
