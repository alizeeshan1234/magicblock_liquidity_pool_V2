"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GetCommitmentSignature = GetCommitmentSignature;
async function GetCommitmentSignature(transactionSignature, ephemeralConnection) {
    const txSchedulingSgn = await ephemeralConnection.getTransaction(transactionSignature, { maxSupportedTransactionVersion: 0 });
    if (txSchedulingSgn?.meta == null) {
        throw new Error("Transaction not found or meta is null");
    }
    const scheduledCommitSgn = parseScheduleCommitsLogsMessage(txSchedulingSgn.meta.logMessages ?? []);
    if (scheduledCommitSgn == null) {
        throw new Error("ScheduledCommitSent signature not found");
    }
    const latestBlockhash = await ephemeralConnection.getLatestBlockhash();
    await ephemeralConnection.confirmTransaction({
        signature: scheduledCommitSgn,
        ...latestBlockhash,
    });
    const txCommitInfo = await ephemeralConnection.getTransaction(scheduledCommitSgn, { maxSupportedTransactionVersion: 0 });
    if (txCommitInfo?.meta == null) {
        throw new Error("Transaction not found or meta is null");
    }
    const commitSignature = parseCommitsLogsMessage(txCommitInfo.meta.logMessages ?? []);
    if (commitSignature == null) {
        throw new Error("Unable to find Commitment signature");
    }
    return commitSignature;
}
function parseScheduleCommitsLogsMessage(logMessages) {
    for (const message of logMessages) {
        const signaturePrefix = "ScheduledCommitSent signature: ";
        if (message.includes(signaturePrefix)) {
            return message.split(signaturePrefix)[1];
        }
    }
    return null;
}
function parseCommitsLogsMessage(logMessages) {
    for (const message of logMessages) {
        const signaturePrefix = "ScheduledCommitSent signature[0]: ";
        if (message.includes(signaturePrefix)) {
            return message.split(signaturePrefix)[1];
        }
    }
    return null;
}
//# sourceMappingURL=utils.js.map