import { exec } from 'child_process';
import { setTimeout } from 'timers/promises';

let number_of_timeouts = 0;
const errors = [];

export const get_total_number_of_timeouts = () => number_of_timeouts;
export const reset_timeouts = () => {
  number_of_timeouts = 0;
};
export const get_errors = () => errors.slice(0);

const command_internal = command => new Promise((resolve, reject) => {
  exec(command, ((error, stdout) => {
    if (error) {
      return reject(error);
    }
    return resolve(stdout.trimEnd());
  }));
});

export const command = async (instructions, retry_if) => {
  while (true) {
    try {
      return await command_internal(instructions);
    } catch (e) {
      number_of_timeouts++;
      errors.push(e.message);
      if (typeof retry_if === 'function') {
        await setTimeout(3000);
        if (await Promise.resolve(retry_if())) {
          continue;
        }
        return null;
      } else {
        throw e;
      }
    }
  }
};
