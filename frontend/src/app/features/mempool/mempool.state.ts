import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { MempoolTransaction } from '@shared/types/mempool/mempool-transaction.type';
import { MempoolFilters } from '@shared/types/mempool/mempool-filters.type';
import { MEMPOOL_KEY } from '@app/features/mempool/mempool.actions';

export interface MempoolState {
  allTxs: MempoolTransaction[];
  txs: MempoolTransaction[];
  activeTx: MempoolTransaction;
  filters: MempoolFilters;
  emptyInDatabase: boolean;
  isLoading: boolean;
}


const select = <T>(selector: (state: MempoolState) => T): MemoizedSelector<MinaState, T> => createSelector(
  createFeatureSelector<MempoolState>(MEMPOOL_KEY),
  selector,
);

const filteredTxs = select(state => state.txs);
const allTxs = select(state => state.allTxs);
const activeTx = select(state => state.activeTx);
const filters = select(state => state.filters);
const emptyInDatabase = select(state => state.emptyInDatabase);
const isLoading = select(state => state.isLoading);

export const MempoolSelectors = {
  filteredTxs,
  allTxs,
  activeTx,
  filters,
  emptyInDatabase,
  isLoading,
};
