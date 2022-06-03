import {
  init_users,
  fund_users,
  run_all_transfers,
  fund_users_wrapped,
  init_wrapped,
  run_all_transfers_wrapped,
} from './users.js';
import timer from './timer.js';
import { readFile, writeFile } from 'fs/promises';
import { fileURLToPath } from 'url';
import * as path from 'path';
import { get_errors, get_total_number_of_timeouts, reset_timeouts } from './dfx.js';
import parameters from './parameters.json' assert { type: 'json' };

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const OUTPUT_FILE = path.resolve(__dirname, 'stats.json');
const NUM_SHARDS = parseInt(process.env.NUM_SHARDS) || 1;

const run = async () => {
  await init_users(parameters.number_of_users);
  console.log('users initialized');

  let funding_time;
  let transfers_time;

  if (NUM_SHARDS === 1) {
    [funding_time] = await timer(() => fund_users());
    console.log('users funded');
    [transfers_time] = await timer(() => run_all_transfers());
    console.log('transfers done');
  } else {
    await init_wrapped();
    [funding_time] = await timer(() => fund_users_wrapped());
    console.log('users funded');
    [transfers_time] = await timer(() => run_all_transfers_wrapped());
    console.log('transfers done');
  }

  let errors = get_errors();
  let e = {};
  errors.forEach(err => {
    err = err.split('\n')[1];
    e[err] = (e[err] || 0) + 1;
  });
  console.log('errors:\n', e);
  let number_of_timeouts = get_total_number_of_timeouts();
  await write_stats(funding_time, transfers_time, number_of_timeouts);
};

const write_stats = async (funding_time, transfers_time, number_of_timeouts) => {
  const num_users = parameters.number_of_users;
  const num_shards = NUM_SHARDS;
  const num_transfers_per_user = parameters.number_of_transactions_per_user;
  const key = `${num_shards};${num_users};${num_transfers_per_user}`;
  const stats = {
    num_shards,
    num_users,
    num_transfers_per_user,
    funding_time,
    transfers_time,
    number_of_timeouts,
  };
  console.log(stats);

  let contents = JSON.parse(
    '' + (await readFile(OUTPUT_FILE)),
  );
  contents[key] = stats;
  await writeFile(OUTPUT_FILE, JSON.stringify(contents, null, 2));
};

run().then(() => console.log('DONE')).catch(e => console.error(`ERROR: ${e.message}`));
