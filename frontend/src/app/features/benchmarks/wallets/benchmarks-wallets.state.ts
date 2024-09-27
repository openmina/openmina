import { BenchmarksWallet } from '@shared/types/benchmarks/wallets/benchmarks-wallet.type';
import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { SentTransactionsStats } from '@shared/types/benchmarks/wallets/sent-transactions-stats.type';
import { BenchmarksWalletTransaction } from '@shared/types/benchmarks/wallets/benchmarks-wallet-transaction.type';
import { selectBenchmarksWalletsState } from '@benchmarks/benchmarks.state';
import { BenchmarksZkapp } from '@shared/types/benchmarks/transactions/benchmarks-zkapp.type';


export interface BenchmarksWalletsState {
  wallets: BenchmarksWallet[];
  blockSending: boolean;
  txSendingBatch: number;
  sentTransactions: SentTransactionsStats;
  sentTxCount: number;
  txsToSend: BenchmarksWalletTransaction[];
  randomWallet: boolean;
  activeWallet: BenchmarksWallet;
  sendingFee: number;
  sendingAmount: number;
  zkAppsSendingBatch: number;
  zkAppsToSend: BenchmarksZkapp[];
  sendingFeeZkapps: number;
}

const select = <T>(selector: (state: BenchmarksWalletsState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectBenchmarksWalletsState,
  selector,
);

export const selectBenchmarksWallets = select((state: BenchmarksWalletsState): BenchmarksWallet[] => state.wallets);
export const selectBenchmarksBlockSending = select((state: BenchmarksWalletsState): boolean => state.blockSending || state.txsToSend.length > 0);
export const selectBenchmarksSentTransactionsStats = select((state: BenchmarksWalletsState): SentTransactionsStats => state.sentTransactions);
export const selectBenchmarksSendingBatch = select((state: BenchmarksWalletsState): number => state.txSendingBatch);
export const selectBenchmarksSendingFee = select((state: BenchmarksWalletsState): number => state.sendingFee);
export const selectBenchmarksSendingAmount = select((state: BenchmarksWalletsState): number => state.sendingAmount);
export const selectBenchmarksRandomWallet = select((state: BenchmarksWalletsState): boolean => state.randomWallet);
export const selectBenchmarksActiveWallet = select((state: BenchmarksWalletsState): BenchmarksWallet => state.activeWallet);
export const selectBenchmarksSendingFeeZkapps = select((state: BenchmarksWalletsState): number => state.sendingFeeZkapps);
export const selectBenchmarksZkappsSendingBatch = select((state: BenchmarksWalletsState): number => state.zkAppsSendingBatch);
