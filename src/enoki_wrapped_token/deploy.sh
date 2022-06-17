. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
if [[ -n "$2" ]]; then
  TOKEN_ID="principal \"$2\""
else
  TOKEN_ID="principal \"$(
    dfx canister id xtc_token
  )\""
fi
dfx deploy $1
dfx canister call $1 finishInit "($TOKEN_ID, \"$3\", \"$4\", \"$5\", $6:nat8, $7)"
