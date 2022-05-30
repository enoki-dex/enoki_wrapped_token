sudo dfx canister --no-wallet create --all
cargo run > enoki_wrapped_token_shard.did
ic-cdk-optimizer target/wasm32-unknown-unknown/release/enoki_wrapped_token_shard.wasm -o target/wasm32-unknown-unknown/release/opt.wasm
sudo dfx build enoki_wrapped_token_shard
OWNER="principal \"$( \
   dfx identity get-principal
)\""
sudo dfx canister --no-wallet install enoki_wrapped_token_shard --argument "(\"test logo\", \"test token\", \"TT\", 8:nat8, 100000000:nat64, $OWNER, 0)" -m=reinstall