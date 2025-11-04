"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.verifyTeeRpcIntegrity = verifyTeeRpcIntegrity;
async function verifyTeeRpcIntegrity(rpcUrl) {
    const { default: init, js_get_collateral: jsGetCollateral, js_verify: jsVerify, } = await import("@phala/dcap-qvl-web");
    const challengeBytes = Buffer.from(Uint8Array.from(Array(32)
        .fill(0)
        .map(() => Math.floor(Math.random() * 256))));
    const challenge = challengeBytes.toString("base64");
    const url = `${rpcUrl}/quote?challenge=${encodeURIComponent(challenge)}`;
    const response = await fetch(url);
    const responseBody = await response.json();
    if (response.status !== 200 || !("quote" in responseBody)) {
        throw new Error(responseBody.error ?? "Failed to get quote");
    }
    await init();
    const rawQuote = Uint8Array.from(Buffer.from(responseBody.quote, "base64"));
    const pccsUrl = "https://pccs.phala.network/tdx/certification/v4";
    const quoteCollateral = await jsGetCollateral(pccsUrl, rawQuote);
    const now = BigInt(Math.floor(Date.now() / 1000));
    try {
        jsVerify(rawQuote, quoteCollateral, now);
        return true;
    }
    catch (error) {
        return false;
    }
}
//# sourceMappingURL=verify.js.map