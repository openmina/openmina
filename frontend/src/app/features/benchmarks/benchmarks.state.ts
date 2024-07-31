import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { BenchmarksWalletsState } from '@benchmarks/wallets/benchmarks-wallets.state';

export interface BenchmarksState {
  wallets: BenchmarksWalletsState;
}

const select = <T>(selector: (state: BenchmarksState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectBenchmarksState,
  selector,
);

export const selectBenchmarksState = createFeatureSelector<BenchmarksState>('benchmarks');
export const selectBenchmarksWalletsState = select((state: BenchmarksState): BenchmarksWalletsState => state.wallets);
