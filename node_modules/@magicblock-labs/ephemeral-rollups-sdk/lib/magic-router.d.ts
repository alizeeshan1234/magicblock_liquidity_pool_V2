import { Connection, Transaction, ConfirmOptions, TransactionSignature, Signer, BlockhashWithExpiryBlockHeight, SendOptions, PublicKey, VersionedTransaction } from "@solana/web3.js";
export declare function getWritableAccounts(transaction: Transaction): string[];
export declare class ConnectionMagicRouter extends Connection {
    getClosestValidator(): Promise<{
        identity: string;
        fqdn?: string;
    }>;
    getDelegationStatus(account: PublicKey | string): Promise<{
        isDelegated: boolean;
    }>;
    getLatestBlockhashForTransaction(transaction: Transaction, options?: ConfirmOptions): Promise<BlockhashWithExpiryBlockHeight>;
    prepareTransaction(transaction: Transaction, options?: ConfirmOptions): Promise<Transaction>;
    sendTransaction(transaction: Transaction | VersionedTransaction, signersOrOptions?: Signer[] | SendOptions, options?: SendOptions): Promise<TransactionSignature>;
    sendAndConfirmTransaction(transaction: Transaction, signers: Signer[], options?: ConfirmOptions & {
        abortSignal?: AbortSignal;
    }): Promise<TransactionSignature>;
}
//# sourceMappingURL=magic-router.d.ts.map