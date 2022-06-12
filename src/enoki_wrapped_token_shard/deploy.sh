. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token_shard.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
if [[ -n "$2" ]]; then
  TOKEN_ID="principal \"$2\""
else
  TOKEN_ID="principal \"$(
    dfx canister id xtc_token
  )\""
fi

i=1
num_shards=${NUM_SHARDS:-2}
while [ $i -le $num_shards ]; do
  dfx build "$1_shard_$i"
  yes yes | dfx canister install "$1_shard_$i" --argument "($3, $TOKEN_ID)" -m=reinstall
  dfx canister call $1 "addShard" "(principal \"$(dfx canister id "$1_shard_$i")\")"
  true $((i++))
done
