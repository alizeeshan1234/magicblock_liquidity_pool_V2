"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ConnectionMagicRouter = void 0;
exports.getWritableAccounts = getWritableAccounts;
const web3_js_1 = require("@solana/web3.js");
function getWritableAccounts(transaction) {
    const writableAccounts = new Set();
    if (transaction.feePayer) {
        writableAccounts.add(transaction.feePayer.toBase58());
    }
    for (const instruction of transaction.instructions) {
        for (const key of instruction.keys) {
            if (key.isWritable) {
                writableAccounts.add(key.pubkey.toBase58());
            }
        }
    }
    return Array.from(writableAccounts);
}
class ConnectionMagicRouter extends web3_js_1.Connection {
    async getClosestValidator() {
        const response = await fetch(this.rpcEndpoint, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                jsonrpc: "2.0",
                id: 1,
                method: "getIdentity",
                params: [],
            }),
        });
        const identityData = (await response.json())?.result;
        if (identityData === null || identityData.identity === undefined) {
            throw new Error("Invalid response");
        }
        return identityData;
    }
    async getDelegationStatus(account) {
        const accountAddress = typeof account === "string" ? account : account.toBase58();
        const response = await fetch(`${this.rpcEndpoint}/getDelegationStatus`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                jsonrpc: "2.0",
                id: 1,
                method: "getDelegationStatus",
                params: [accountAddress],
            }),
        });
        return (await response.json()).result;
    }
    async getLatestBlockhashForTransaction(transaction, options) {
        const writableAccounts = getWritableAccounts(transaction);
        const blockHashResponse = await fetch(this.rpcEndpoint, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                jsonrpc: "2.0",
                id: 1,
                method: "getBlockhashForAccounts",
                params: [writableAccounts],
            }),
        });
        const blockHashData = await blockHashResponse.json();
        return blockHashData.result;
    }
    async prepareTransaction(transaction, options) {
        const blockHashData = await this.getLatestBlockhashForTransaction(transaction, options);
        transaction.recentBlockhash = blockHashData.blockhash;
        return transaction;
    }
    async sendTransaction(transaction, signersOrOptions, options) {
        if (transaction instanceof web3_js_1.Transaction) {
            const latestBlockhash = await this.getLatestBlockhashForTransaction(transaction);
            transaction.recentBlockhash = latestBlockhash.blockhash;
            transaction.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;
            if (Array.isArray(signersOrOptions)) {
                transaction.sign(...signersOrOptions);
            }
            const wireTx = transaction.serialize();
            return this.sendRawTransaction(wireTx, options);
        }
        else {
            return super.sendTransaction(transaction, signersOrOptions);
        }
    }
    async sendAndConfirmTransaction(transaction, signers, options) {
        const signature = await this.sendTransaction(transaction, signers, options);
        let status;
        const { recentBlockhash, lastValidBlockHeight, minNonceContextSlot, nonceInfo, } = transaction;
        if (recentBlockhash !== undefined && lastValidBlockHeight !== undefined) {
            status = (await this.confirmTransaction({
                abortSignal: options?.abortSignal,
                signature,
                blockhash: recentBlockhash,
                lastValidBlockHeight,
            }, options?.commitment)).value;
        }
        else if (minNonceContextSlot !== undefined && nonceInfo !== undefined) {
            const { nonceInstruction } = nonceInfo;
            const nonceAccountPubkey = nonceInstruction.keys[0].pubkey;
            status = (await this.confirmTransaction({
                abortSignal: options?.abortSignal,
                minContextSlot: minNonceContextSlot,
                nonceAccountPubkey,
                nonceValue: nonceInfo.nonce,
                signature,
            }, options?.commitment)).value;
        }
        else {
            status = (await this.confirmTransaction(signature, options?.commitment))
                .value;
        }
        if (status.err != null) {
            throw new web3_js_1.SendTransactionError({
                action: "send",
                signature,
                transactionMessage: `Status: (${JSON.stringify(status)})`,
            });
        }
        return signature;
    }
}
exports.ConnectionMagicRouter = ConnectionMagicRouter;
//# sourceMappingURL=magic-router.js.map