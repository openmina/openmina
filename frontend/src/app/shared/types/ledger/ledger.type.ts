/**
 * Transport mechanism.
 * @see https://github.com/LedgerHQ/ledgerjs for all transport Available
 */
export interface Transport {
  /**
   * Used in U2F to avoid different web apps to communicate with different ledger implementations
   * @param {string} key
   */
  setScrambleKey(key: string): void;

  /**
   * sends data to Ledger
   * @param {number} cla
   * @param {number} ins
   * @param {number} p1
   * @param {number} p2
   * @param {Buffer} data
   * @param {number[]} statusList Allowed status list.
   * @returns {Promise<Buffer>}
   */
  send(
    cla: number,
    ins: number,
    p1: number,
    p2: number,
    data?: Buffer,
    statusList?: number[],
  ): Promise<Buffer>;
}

export interface SignTransactionArgs {
  txType: number;
  senderAccount: number;
  senderAddress: string;
  receiverAddress: string;
  amount: number;
  fee: number;
  nonce: number;
  validUntil?: number;
  memo?: string;
  networkId: number;
}

export interface BaseLedgerResponse {
  returnCode: string;
  statusText?: string;
  message?: string;
}

export interface GetAddressResponse extends BaseLedgerResponse {
  publicKey?: string;
}

export interface SignTransactionResponse extends BaseLedgerResponse {
  signature?: string;
}

export interface GetAppVersionResponse extends BaseLedgerResponse {
  version?: string;

}

export interface GetAppNameResponse extends BaseLedgerResponse {
  name?: string;
  version?: string;
}

export enum Networks {
  MAINNET = 0x01,
  DEVNET = 0x00,
}

export enum TxType {
  PAYMENT = 0x00,
  DELEGATION = 0x04,
}
