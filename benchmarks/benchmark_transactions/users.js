import {command} from './dfx.js';
import {parse_candid} from './utils.js';
import {readFileSync} from "fs";
import {fileURLToPath} from 'url';
import * as path from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const parametersStr = readFileSync(path.resolve(__dirname, 'parameters.json'));
const parameters = JSON.parse(parametersStr);

const FUNDING_AMOUNT_PER_USER = '9_000_000_000';
const FUNDING_AMOUNT_TOTAL = '9_990_000_000_000_000';
const TRANSACTION_AMOUNT = '1_000_000';

// const NETWORK = "";
// const NETWORK = " --network local";
const NETWORK = " --network ic";

const users = [];

class User {
    constructor(id) {
        this.id = id;
        this.name = `user${id}`;
        this.principal = '';
        this.balance = '0';
        this.assigned_shard = '';
        this.shard_principal = '';
    }

    async init() {
        await command(`dfx identity${NETWORK} new ${this.name} || true`);
        this.principal = await command(`dfx --identity ${this.name} identity${NETWORK} get-principal`);
    }

    async exec(instruction, retry_if) {
        if (NETWORK) {
            let inst = instruction.split(' ');
            inst.splice(1, 0, NETWORK.trim());
            instruction = inst.join(' ');
        }
        return await command(`dfx --identity ${this.name} ${instruction}`, retry_if);
    }

    get_random_counterpart() {
        let id = this.id;
        while (id === this.id) {
            id = Math.floor(Math.random() * users.length);
        }
        return id;
    }

    async run_transfers() {
        for (let i = 0; i < parameters.number_of_transactions_per_user; i++) {
            let counterpart = users[this.get_random_counterpart()];
            let old_balance = this.balance;
            await this.exec(`canister call xtc_token transfer '(principal "${counterpart.principal}",${TRANSACTION_AMOUNT})'`, async () => {
                await this.update_balance();
                return this.balance === old_balance;
            });
            await this.update_balance();
        }
    }

    async update_balance() {
        let response = await command(`dfx canister${NETWORK} call xtc_token balanceOf '(principal "${this.principal}")'`, () => true);
        this.balance = parse_candid(response)[0][0];
    }

    async update_shard_balance() {
        let response = await command(`dfx canister${NETWORK} call ${this.assigned_shard} shardBalanceOf '(principal "${this.shard_principal}")'`, () => true);
        this.balance = parse_candid(response)[0][0];
    }

    async fund() {
        let old_balance = this.balance;
        await command(`dfx canister${NETWORK} call xtc_token transfer '(principal "${this.principal}",${FUNDING_AMOUNT_PER_USER})'`, async () => {
            await this.update_balance();
            return this.balance === old_balance;
        });
        await this.update_balance();
    }

    async init_wrapped() {
        this.assigned_shard = parse_candid(await this.exec(`canister call enoki_wrapped_token startRegistration`))[0][0];
        this.shard_principal = parse_candid(await this.exec(`canister call ${this.assigned_shard} whoami`))[0][0];
        await this.exec(`canister call enoki_wrapped_token completeRegistration '(principal "${this.shard_principal}")'`);
    }

    async fund_wrapped() {
        let old_balance = this.balance;
        await command(`dfx canister${NETWORK} call ${default_assigned_shard} shardTransfer '(principal "${this.assigned_shard}", principal "${this.shard_principal}",${FUNDING_AMOUNT_PER_USER})'`, async () => {
            await this.update_shard_balance();
            return this.balance === old_balance;
        });
        await this.update_shard_balance();
    }

    async run_transfers_wrapped() {
        for (let i = 0; i < parameters.number_of_transactions_per_user; i++) {
            let counterpart = users[this.get_random_counterpart()];
            let old_balance = this.balance;
            await this.exec(`canister call ${this.assigned_shard} shardTransfer '(principal "${counterpart.assigned_shard}", principal "${counterpart.shard_principal}",${TRANSACTION_AMOUNT})'`, async () => {
                await this.update_shard_balance();
                return this.balance === old_balance;
            });
            await this.update_shard_balance();
        }
    }
}

User.new = async () => {
    const id = users.length;
    const user = new User(id);
    users.push(user);
    await user.init();
    return user;
};

const prepare_fund_users = async () => {
    await command(`dfx identity${NETWORK} use default`);
};

export const init_users = async count => {
    const promises = [];
    for (let i = 0; i < count; i++) {
        promises.push(User.new());
    }
    await Promise.all(promises);
};

export const fund_users = async () => {
    await prepare_fund_users();
    const promises = [];
    users.forEach(user => {
        promises.push(user.fund());
    });
    await Promise.all(promises);
};

export const run_all_transfers = async () => {
    const promises = [];
    users.forEach(user => {
        promises.push(user.run_transfers());
    });
    await Promise.all(promises);
    console.log('users: ', users);
};

let default_shard_principal = '';
let default_assigned_shard = '';

export const init_wrapped = async () => {
    await command(`dfx identity${NETWORK} use default`);
    default_assigned_shard = parse_candid(await command(`dfx canister${NETWORK} call enoki_wrapped_token startRegistration`))[0][0];
    console.log(`assigned: ${default_assigned_shard}`);
    default_shard_principal = parse_candid(await command(`dfx canister${NETWORK} call ${default_assigned_shard} whoami`))[0][0];
    console.log(`me: ${default_shard_principal}`);
    await command(`dfx canister${NETWORK} call enoki_wrapped_token completeRegistration '(principal "${default_shard_principal}")'`);

    let n = 50;
    for (let i = 0; i * n < users.length; i++) {
        const promises = [];
        for (let j = n * i; j < users.length && j < n * (i + 1); j++) {
            promises.push(users[j].init_wrapped());
        }
        await Promise.all(promises);
    }
}

export const fund_users_wrapped = async () => {
    await command(`dfx canister${NETWORK} call xtc_token approve '(principal "${default_assigned_shard}", ${FUNDING_AMOUNT_TOTAL})'`);
    await command(`dfx canister${NETWORK} call ${default_assigned_shard} wrap "(${FUNDING_AMOUNT_TOTAL})"`);

    const promises = [];
    users.forEach(user => {
        promises.push(user.fund_wrapped());
    });
    await Promise.all(promises);
    // console.log('users: ', users);
};

export const run_all_transfers_wrapped = async () => {
    const promises = [];
    users.forEach(user => {
        promises.push(user.run_transfers_wrapped());
    });
    await Promise.all(promises);
    // console.log('users: ', users);
};
