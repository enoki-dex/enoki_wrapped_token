const fs = require('fs');

const number_of_shards = parseInt(process.env.NUM_SHARDS) || 2;

const config = JSON.parse('' + fs.readFileSync('./dfx.1.json'));
const shard_config = config['canisters']['enoki_wrapped_token_shard_1'];
const shard_config_b = config['canisters']['enoki_wrapped_token_b_shard_1'];
for (let i = 2; i <= number_of_shards; i++) {
    config['canisters'][`enoki_wrapped_token_shard_${i}`] = shard_config;
    if (shard_config_b) {
        config['canisters'][`enoki_wrapped_token_b_shard_${i}`] = shard_config_b;
    }
}

fs.writeFileSync('./dfx.json', JSON.stringify(config, null, 2));