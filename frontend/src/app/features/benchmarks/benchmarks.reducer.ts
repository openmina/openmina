import { BenchmarksState } from '@benchmarks/benchmarks.state';
import { ActionReducer, combineReducers } from '@ngrx/store';

import * as fromWallets from '@benchmarks/wallets/benchmarks-wallets.reducer';


export const benchmarksReducer: ActionReducer<BenchmarksState, any> = combineReducers<BenchmarksState, any>({
  wallets: fromWallets.reducer,
});
