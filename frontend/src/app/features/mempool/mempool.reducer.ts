import { createReducer, on } from '@ngrx/store';
import { MempoolState } from '@app/features/mempool/mempool.state';
import { MempoolActions } from '@app/features/mempool/mempool.actions';
import { MempoolTransaction, MempoolTransactionKind } from '@shared/types/mempool/mempool-transaction.type';
import { MempoolFilters } from '@shared/types/mempool/mempool-filters.type';

const initialState: MempoolState = {
  allTxs: [],
  txs: [],
  activeTx: null,
  emptyInDatabase: false,
  isLoading: true,
  filters: {
    delegation: true,
    payment: true,
    zkApp: true,
    search: '',
  },
};

export const mempoolReducer = createReducer(
  initialState,
  on(MempoolActions.getTxsSuccess, (state, { txs }) => ({
    ...state,
    emptyInDatabase: txs.length === 0,
    isLoading: false,
    allTxs: txs,
    txs: filterTxs(txs, state.filters),
  })),
  on(MempoolActions.changeFilters, (state, { filters }) => ({
    ...state,
    filters,
    txs: filterTxs(state.allTxs, filters),
  })),
  on(MempoolActions.setActiveTx, (state, { tx }) => ({ ...state, activeTx: tx })),
  on(MempoolActions.close, () => initialState),
);

function filterTxs(txs: MempoolTransaction[], filters: MempoolFilters): MempoolTransaction[] {
  return txs.filter(tx => {
    if (filters.delegation === false && tx.kind === MempoolTransactionKind.DELEGATION) return false;
    if (filters.payment === false && tx.kind === MempoolTransactionKind.PAYMENT) return false;
    if (filters.zkApp === false && tx.kind === MempoolTransactionKind.ZK_APP) return false;

    if (filters.search?.length > 2) {
      const searchTerm = filters.search.toLowerCase();
      const searchMatch = tx.txHash.toLowerCase().includes(searchTerm) || tx.sender.toLowerCase().includes(searchTerm);
      if (!searchMatch) return false;
    }

    return true;
  });
}
