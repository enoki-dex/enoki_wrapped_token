. "$(dirname "$0")"/setup.sh

start "setup"
"$(dirname "$0")"/mock_exchange/deploy.sh
end

start "creating users"
dfx identity new user1 || true
dfx identity new user2 || true
dfx identity use user1
USER1=$(dfx identity get-principal)
info "user1: $USER1"
dfx identity use user2
USER2=$(dfx identity get-principal)
info "user2: $USER2"

dfx identity use default
info "balance of default: $(dfx canister call xtc_token balanceOf "(principal \"$(dfx identity get-principal)\")")"
dfx canister call xtc_token transfer "(principal \"$USER1\", 12345678000)"
info "balance of user1: $(dfx canister call xtc_token balanceOf "(principal \"$USER1\")")"
end

start "setting up user1 on enoki_wrapped_token"
dfx identity use user1
ASSIGNED_SHARD=$(dfx canister call enoki_wrapped_token register "(principal \"$USER1\")" | grep -oE $REGEX_PRINCIPAL)
info "user1 assigned to: $ASSIGNED_SHARD"
dfx canister call xtc_token approve "(principal \"$ASSIGNED_SHARD\", 12300000000)"
info "wrapping original token"
dfx canister call "$ASSIGNED_SHARD" wrap "(12300000000)"
info "user1 balance original token: $(dfx canister call xtc_token balanceOf "(principal \"$USER1\")")"
info "user1 balance of wrapped token: $(dfx canister call enoki_wrapped_token balanceOf "(principal \"$USER1\")")"
info "total supply of wrapped token: $(dfx canister call enoki_wrapped_token totalSupply)"
end

start "deposit to exchange"
DEPOSIT_SHARD=$(dfx canister call mock_exchange getDepositShardId | grep -oE $REGEX_PRINCIPAL)
EXCHANGE_ID=$(dfx canister id mock_exchange)
dfx canister call "$ASSIGNED_SHARD" shardTransferAndCall "(principal \"$DEPOSIT_SHARD\", principal \"$EXCHANGE_ID\", 1220000000, principal \"$EXCHANGE_ID\", \"deposit\", \"\")"
BALANCE=$(dfx canister call mock_exchange balance)
info "user1 balance on exchange: $BALANCE"
assert_eq "$BALANCE" "(1_219_980_000 : nat)"
info "user1 balance of wrapped token: $(dfx canister call enoki_wrapped_token balanceOf "(principal \"$USER1\")")"
info "total supply of wrapped token: $(dfx canister call enoki_wrapped_token totalSupply)"
info "total accrued fees of wrapped token: $(dfx canister call enoki_wrapped_token getAccruedFees)"
end

start "unwrap token"
info "withdrawing from exchange"
dfx canister call mock_exchange withdrawAll "(principal \"$ASSIGNED_SHARD\", principal \"$USER1\")"
info "user1 balance on exchange: $(dfx canister call mock_exchange balance)"
info "user1 balance of wrapped token: $(dfx canister call enoki_wrapped_token balanceOf "(principal \"$USER1\")")"
AMOUNT="11_079_999_580"
info "unwrapping $AMOUNT tokens"
dfx canister call "$ASSIGNED_SHARD" unwrap "($AMOUNT, principal \"$USER1\")"
info "user1 balance original token: $(dfx canister call xtc_token balanceOf "(principal \"$USER1\")")"
info "user1 balance of wrapped token: $(dfx canister call enoki_wrapped_token balanceOf "(principal \"$USER1\")")"
info "total supply of wrapped token: $(dfx canister call enoki_wrapped_token totalSupply)"
info "total accrued fees of wrapped token: $(dfx canister call enoki_wrapped_token getAccruedFees)"
end
