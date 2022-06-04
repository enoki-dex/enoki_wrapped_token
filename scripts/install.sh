set -a # automatically export all variables
source .env
set +a

## === INSTALL APP ====
dfx identity use default
./src/enoki_wrapped_token/deploy.sh enoki_wrapped_token "$UNDERLYING_TOKEN_ID_A" "$TOKEN_LOGO_A" "$TOKEN_NAME_A" "$TOKEN_SYMBOL_A" "$TOKEN_DECIMALS_A" "$TOKEN_FEE_A"
MANAGER_ID="principal \"$(
  dfx canister id enoki_wrapped_token
)\""
./src/enoki_wrapped_token_shard/deploy.sh enoki_wrapped_token "$UNDERLYING_TOKEN_ID_A" "$MANAGER_ID"

if [ -n "$DEPLOY_TOKEN_B" ]; then
  ./src/enoki_wrapped_token/deploy.sh enoki_wrapped_token_b "$UNDERLYING_TOKEN_ID_B" "$TOKEN_LOGO_B" "$TOKEN_NAME_B" "$TOKEN_SYMBOL_B" "$TOKEN_DECIMALS_B" "$TOKEN_FEE_B"
  MANAGER_ID="principal \"$(
    dfx canister id enoki_wrapped_token_b
  )\""
  ./src/enoki_wrapped_token_shard/deploy.sh enoki_wrapped_token_b "$UNDERLYING_TOKEN_ID_B" "$MANAGER_ID"
fi
