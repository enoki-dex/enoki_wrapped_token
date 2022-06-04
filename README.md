# Enoki Token Standard

The Enoki Wrapped Token is a proof of concept for a scalable fungible token on the Internet Computer. It implements sharding by account in order to run transactions in parallel. To further optimize performance, users can choose to move to less utilised shards.

It was created specifically for use in Enoki Exchange. It is very minimalistic (ex: does not keep transaction history) and subject to many modifications pending community consensus on a token standard.

It is loosely based on the DIP20 Token Standard. It contains all DIP20 methods, with some caveats:
- `name`, `symbol`, `getLogo`, `balanceOf`, etc, work as expected.
- `transfer` is slow because it involves several canister calls.
  - `shardTransfer` should be used instead, which is called at the shard contract (and not the main contract).
- `approve` and `transferFrom` will always fail, since this token standard uses subscriptions (aka notifications), and not approvals, for inter-contract calls. 
  - `transferAndCall` (slow) and `shardTransferAndCall` (preferred) should be used instead.

# Development

## Dependencies

- [dfx](https://smartcontracts.org/docs/developers-guide/install-upgrade-remove.html)
- [cmake](https://cmake.org/)

[//]: # (- [npm]&#40;https://nodejs.org/en/download/&#41;)

Make sure you have wasm as a target:
```
rustup target add wasm32-unknown-unknown
```

## Local Deploy

### Configure

```bash
cp default.env .env
```
If you make any changes to `.env`, please run:
```bash
make config
```

### Run
```bash
make deps && make install
```
to stop and reset the local data:
```bash
make clean
```

[//]: # (The app's local URL should be displayed. When you log in, your principal will be displayed.)

[//]: # (Give yourself tokens by running:)

[//]: # ()
[//]: # (```bash)

[//]: # (make init-local II_PRINCIPAL=<YOUR II PRINCIPAL>)

[//]: # (```)

### Test
```bash
make test
```

# Pending Features

- use tokens collected by this contract in fees in some sort of auction to have users refill cycles.
- shard contains a max number of accounts (as an upper limit for memory)
- account data includes the user's Principal on the main canister (to allow for scale up/down)
- scale up automatically (maybe use big-map as a model: https://github.com/dfinity/bigmap-poc)
- how to scale down? It probably needs users to make a couple of transactions.
- function for a user to call to change to a less-used (faster) shard
- change hashmap of all user accounts to a big-map
- transaction history is kept by an archive canister (using a big-map) that listens (PubSub) to all transactions for all shards
