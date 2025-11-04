"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const vitest_1 = require("vitest");
const magic_router_js_1 = require("../magic-router.js");
const web3_js_1 = require("@solana/web3.js");
global.fetch = vitest_1.vi.fn();
(0, vitest_1.describe)("getWritableAccounts", () => {
    const mockPublicKey = (address) => ({
        toBase58: () => address,
        toString: () => address,
    });
    (0, vitest_1.it)("deduplicates writable accounts", () => {
        const tx = {
            feePayer: mockPublicKey("fee-payer"),
            instructions: [
                {
                    keys: [
                        { pubkey: mockPublicKey("k1"), isWritable: true },
                        { pubkey: mockPublicKey("k1"), isWritable: true },
                    ],
                },
            ],
        };
        const result = (0, magic_router_js_1.getWritableAccounts)(tx);
        (0, vitest_1.expect)(result).toEqual(["fee-payer", "k1"]);
    });
});
(0, vitest_1.describe)("Connection prototype methods", () => {
    let connection;
    let tx;
    (0, vitest_1.beforeEach)(() => {
        connection = new magic_router_js_1.ConnectionMagicRouter("http://localhost");
        tx = new web3_js_1.Transaction();
        tx.serialize = vitest_1.vi.fn(() => Buffer.from("mock"));
        tx.sign = vitest_1.vi.fn();
        global.fetch.mockReset();
    });
    (0, vitest_1.it)("getClosestValidator returns identity", async () => {
        global.fetch.mockResolvedValueOnce({
            json: async () => ({ result: { identity: "validator-1" } }),
        });
        const result = await connection.getClosestValidator();
        (0, vitest_1.expect)(result).toEqual({ identity: "validator-1" });
    });
    (0, vitest_1.it)("getDelegationStatus works with string account", async () => {
        global.fetch.mockResolvedValueOnce({
            json: async () => ({ result: { isDelegated: true } }),
        });
        const result = await connection.getDelegationStatus("account1");
        (0, vitest_1.expect)(result).toEqual({ isDelegated: true });
    });
    (0, vitest_1.it)("getDelegationStatus works with PublicKey account", async () => {
        global.fetch.mockResolvedValueOnce({
            json: async () => ({ result: { isDelegated: false } }),
        });
        const pk = new web3_js_1.PublicKey("11111111111111111111111111111111");
        const result = await connection.getDelegationStatus(pk);
        (0, vitest_1.expect)(result).toEqual({ isDelegated: false });
    });
    (0, vitest_1.it)("getLatestBlockhashForTransaction returns blockhash", async () => {
        global.fetch.mockResolvedValueOnce({
            json: async () => ({
                result: { blockhash: "mock-blockhash", lastValidBlockHeight: 100 },
            }),
        });
        const result = await connection.getLatestBlockhashForTransaction(tx);
        (0, vitest_1.expect)(result).toEqual({
            blockhash: "mock-blockhash",
            lastValidBlockHeight: 100,
        });
    });
    (0, vitest_1.it)("prepareTransaction sets recentBlockhash", async () => {
        vitest_1.vi.spyOn(magic_router_js_1.ConnectionMagicRouter.prototype, "getLatestBlockhashForTransaction").mockResolvedValue({
            blockhash: "hb",
            lastValidBlockHeight: 100,
        });
        const result = await connection.prepareTransaction(tx);
        (0, vitest_1.expect)(result.recentBlockhash).toBe("hb");
    });
    (0, vitest_1.it)("sendTransaction signs and sends transaction", async () => {
        vitest_1.vi.spyOn(magic_router_js_1.ConnectionMagicRouter.prototype, "getLatestBlockhashForTransaction").mockResolvedValue({
            blockhash: "hb",
            lastValidBlockHeight: 100,
        });
        vitest_1.vi.spyOn(magic_router_js_1.ConnectionMagicRouter.prototype, "sendRawTransaction").mockResolvedValue("sig123");
        const signers = [new web3_js_1.Keypair()];
        const sendTx = connection.sendTransaction.bind(connection);
        const signature = await sendTx(tx, signers);
        const signFn = tx.sign.bind(tx);
        const serializeFn = tx.serialize.bind(tx);
        (0, vitest_1.expect)(signFn(...signers)).toBeUndefined();
        (0, vitest_1.expect)(serializeFn()).toBeInstanceOf(Buffer);
        (0, vitest_1.expect)(signature).toBe("sig123");
    });
    (0, vitest_1.it)("sendAndConfirmTransaction calls sendTransaction and returns signature", async () => {
        vitest_1.vi.spyOn(magic_router_js_1.ConnectionMagicRouter.prototype, "sendTransaction").mockResolvedValue("sig123");
        vitest_1.vi.spyOn(magic_router_js_1.ConnectionMagicRouter.prototype, "confirmTransaction").mockResolvedValue({ value: { err: null } });
        const signature = await (0, web3_js_1.sendAndConfirmTransaction)(connection, tx, [
            new web3_js_1.Keypair(),
        ]);
        (0, vitest_1.expect)(signature).toBe("sig123");
    });
    (0, vitest_1.it)("sendAndConfirmTransaction throws SendTransactionError if status has err", async () => {
        vitest_1.vi.spyOn(magic_router_js_1.ConnectionMagicRouter.prototype, "sendTransaction").mockResolvedValue("sig123");
        vitest_1.vi.spyOn(magic_router_js_1.ConnectionMagicRouter.prototype, "confirmTransaction").mockResolvedValue({ value: { err: { some: "error" } } });
        await (0, vitest_1.expect)((0, web3_js_1.sendAndConfirmTransaction)(connection, tx, [new web3_js_1.Keypair()])).rejects.toThrow(web3_js_1.SendTransactionError);
    });
});
//# sourceMappingURL=magic-router.test.js.map