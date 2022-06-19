dfx identity use default
dfx canister create --all

### === DEPLOY LOCAL LEDGER =====
dfx identity new minter
dfx identity use minter
export MINT_ACC=$(dfx ledger account-id)

dfx identity use default
export LEDGER_ACC=$(dfx ledger account-id)

# Use private api for install
rm src_dev/ledger/ledger.did
cp src_dev/ledger/ledger.private.did src_dev/ledger/ledger.did

dfx deploy ledger --argument '(record  {
    minting_account = "'${MINT_ACC}'";
    initial_values = vec { record { "'${LEDGER_ACC}'"; record { e8s=100_000_000_000 } }; };
    send_whitelist = vec {}
    })'
export LEDGER_ID=$(dfx canister id ledger)

# Replace with public api
rm src_dev/ledger/ledger.did
cp src_dev/ledger/ledger.public.did src_dev/ledger/ledger.did

### === DEPLOY DIP TOKENS =====

dfx canister create xtc_token
dfx build xtc_token

export ROOT_PRINCIPAL="principal \"$(dfx identity get-principal)\""
dfx canister install xtc_token --argument="(\"https://dank.ooo/images/dfinity-gradient-p-500.png\", \"Cycles Token\", \"XTC\", 12, 10000000000000000000, $ROOT_PRINCIPAL, 10000)"

# set fees
#dfx canister call xtc_token setFeeTo "($ROOT_PRINCIPAL)"
dfx canister call xtc_token setFee "(420)"

### === DEPLOY INTERNET IDENTITY =====

II_ENV=development dfx deploy internet_identity --argument '(null)'

