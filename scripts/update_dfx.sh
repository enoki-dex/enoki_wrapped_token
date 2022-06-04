set -a # automatically export all variables
source .env
set +a

node scripts/generate_dfx_json.js
