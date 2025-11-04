import { PublicKey, Connection, Transaction } from "@solana/web3.js";
export interface Configuration {
    chain: string;
    websocket: string;
}
declare enum DelegationStatus {
    Delegated = 0,
    Undelegated = 1
}
type DelegationRecord = {
    status: DelegationStatus.Delegated;
    validator: PublicKey;
} | {
    status: DelegationStatus.Undelegated;
};
export declare class Resolver {
    private readonly routes;
    private readonly delegations;
    private readonly chain;
    private readonly ws;
    private readonly subs;
    constructor(config: Configuration, routes: Map<string, string>);
    trackAccount(pubkey: PublicKey): Promise<DelegationRecord>;
    resolve(pubkey: PublicKey): Promise<Connection | undefined>;
    resolveForTransaction(tx: Transaction): Promise<Connection | undefined>;
    terminate(): Promise<void>;
    private updateStatus;
}
export {};
//# sourceMappingURL=resolver.d.ts.map