dfx start --background --clean --host 127.0.0.1:8000

### === DEPLOY LOCAL LEDGER =====
dfx identity new minter
dfx identity use minter
export MINT_ACC=$(dfx ledger account-id)

dfx identity use default
export LEDGER_ACC=$(dfx ledger account-id)

# Use private api for install
rm src/ledger/ledger.did
cp src/ledger/ledger.private.did src/ledger/ledger.did

dfx deploy ledger --argument '(record  {
    minting_account = "'${MINT_ACC}'";
    initial_values = vec { record { "'${LEDGER_ACC}'"; record { e8s=100_000_000_000 } }; };
    send_whitelist = vec {}
    })'
export LEDGER_ID=$(dfx canister id ledger)

# Replace with public api
rm src/ledger/ledger.did
cp src/ledger/ledger.public.did src/ledger/ledger.did

### === DEPLOY DIP TOKENS =====

dfx canister create xtc_token
dfx build xtc_token

export ROOT_PRINCIPAL="principal \"$(dfx identity get-principal)\""
dfx canister install xtc_token --argument="(\"https://dank.ooo/images/dfinity-gradient-p-500.png\", \"Cycles Token\", \"XTC\", 12, 10000000000000000, $ROOT_PRINCIPAL, 10000)"

# set fees
#dfx canister call xtc_token setFeeTo "($ROOT_PRINCIPAL)"
dfx canister call xtc_token setFee "(420)"

### === DEPLOY INTERNET IDENTITY =====

II_ENV=development dfx deploy internet_identity --no-wallet --argument '(null)'

## === INSTALL APP ====

pushd ./src/enoki_wrapped_token
./build.sh
popd
pushd ./src/enoki_wrapped_token_shard
./build.sh
popd
#dfx deploy enoki_wrapped_token
#dfx deploy enoki_wrapped_token_shard
