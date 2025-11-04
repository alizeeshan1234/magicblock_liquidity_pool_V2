"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Resolver = void 0;
const web3_js_1 = require("@solana/web3.js");
const constants_js_1 = require("./constants.js");
var DelegationStatus;
(function (DelegationStatus) {
    DelegationStatus[DelegationStatus["Delegated"] = 0] = "Delegated";
    DelegationStatus[DelegationStatus["Undelegated"] = 1] = "Undelegated";
})(DelegationStatus || (DelegationStatus = {}));
class Resolver {
    constructor(config, routes) {
        this.routes = new Map();
        this.delegations = new Map();
        this.subs = new Set();
        this.chain = new web3_js_1.Connection(config.chain);
        this.ws = new web3_js_1.Connection(config.websocket);
        this.routes = new Map([...routes.entries()].map(([k, v]) => [k, new web3_js_1.Connection(v)]));
    }
    async trackAccount(pubkey) {
        const pubkeyStr = pubkey.toString();
        if (this.delegations.has(pubkeyStr)) {
            const record = this.delegations.get(pubkeyStr);
            if (record !== undefined) {
                return record;
            }
            throw new Error(`Expected a delegation record for ${pubkeyStr}, but found undefined.`);
        }
        const seed = new TextEncoder().encode("delegation");
        const seeds = [seed, pubkey.toBytes()];
        const [delegationRecord] = web3_js_1.PublicKey.findProgramAddressSync(seeds, constants_js_1.DELEGATION_PROGRAM_ID);
        const id = this.ws.onAccountChange(delegationRecord, (acc) => this.updateStatus(acc, pubkey), "confirmed");
        this.subs.add(id);
        const accountInfo = await this.chain.getAccountInfo(delegationRecord, "confirmed");
        return this.updateStatus(accountInfo, pubkey);
    }
    async resolve(pubkey) {
        let record = this.delegations.get(pubkey.toString());
        if (!record) {
            record = await this.trackAccount(pubkey);
        }
        return record.status === DelegationStatus.Delegated
            ? this.routes.get(record.validator.toString())
            : this.chain;
    }
    async resolveForTransaction(tx) {
        const validators = new Set();
        for (const { pubkey, isWritable } of tx.instructions.flatMap((i) => i.keys)) {
            if (!isWritable)
                continue;
            const record = await this.trackAccount(pubkey);
            if (record.status === DelegationStatus.Delegated) {
                validators.add(record.validator.toString());
            }
        }
        const vs = [...validators];
        return vs.length === 1
            ? this.routes.get(vs[0])
            : validators.size === 0
                ? this.chain
                : undefined;
    }
    async terminate() {
        await Promise.all([...this.subs].map(async (sub) => this.ws.removeAccountChangeListener(sub)));
    }
    updateStatus(account, pubkey) {
        const isDelegated = account !== null &&
            account.owner.equals(constants_js_1.DELEGATION_PROGRAM_ID) &&
            account.lamports !== 0;
        const record = isDelegated
            ? {
                status: DelegationStatus.Delegated,
                validator: new web3_js_1.PublicKey(account.data.subarray(8, 40)),
            }
            : { status: DelegationStatus.Undelegated };
        this.delegations.set(pubkey.toString(), record);
        return record;
    }
}
exports.Resolver = Resolver;
//# sourceMappingURL=resolver.js.map