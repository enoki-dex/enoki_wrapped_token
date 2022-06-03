. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token_shard.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
dfx build enoki_wrapped_token_shard
dfx build enoki_wrapped_token_shard_2
OWNER="principal \"$(
  dfx identity get-principal
)\""
TOKEN_ID="principal \"$(
  dfx canister id xtc_token
)\""
MANAGER_ID="principal \"$(
  dfx canister id enoki_wrapped_token
)\""

i=1
num_shards=${NUM_SHARDS:-1}
while [ $i -le $num_shards ]; do
  yes yes | dfx canister install "enoki_wrapped_token_shard_$i" --argument "($OWNER, $MANAGER_ID, $TOKEN_ID)" -m=reinstall
  dfx canister call enoki_wrapped_token "addShard" "(principal \"$(dfx canister id "enoki_wrapped_token_shard_$i")\")"
  true $((i++))
done
