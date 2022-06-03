. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
dfx build enoki_wrapped_token
OWNER="principal \"$( \
   dfx identity get-principal
)\""
TOKEN_ID="principal \"$( \
   dfx canister id xtc_token
)\""
yes yes | dfx canister install enoki_wrapped_token --argument "($TOKEN_ID, \"test logo\", \"enoki-boosed XTC\", \"eXTC\", 12:nat8, $OWNER, 20000)" -m=reinstall