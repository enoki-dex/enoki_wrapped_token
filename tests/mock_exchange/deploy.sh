. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/mock_exchange.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
dfx build mock_exchange
TOKEN_ID="principal \"$( \
   dfx canister id enoki_wrapped_token
)\""
dfx canister install mock_exchange --argument "($TOKEN_ID)" -m=reinstall
dfx canister call mock_exchange initialize