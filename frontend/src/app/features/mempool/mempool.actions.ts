import { createType } from '@shared/constants/store-functions';
import { createAction, props } from '@ngrx/store';
import { MempoolTransaction } from '@shared/types/mempool/mempool-transaction.type';
import { MempoolFilters } from '@shared/types/mempool/mempool-filters.type';

export const MEMPOOL_KEY = 'mempool';
export const MEMPOOL_PREFIX = 'Mempool';

const type = <T extends string>(type: T) => createType(MEMPOOL_PREFIX, null, type);

const init = createAction(type('Init'));
const close = createAction(type('Close'));
const getTxs = createAction(type('Get Txs'));
const getTxsSuccess = createAction(type('Get Txs Success'), props<{
  txs: MempoolTransaction[],
}>());
const changeFilters = createAction(type('Change Filters'), props<{ filters: MempoolFilters }>());
const setActiveTx = createAction(type('Set Active Tx'), props<{ tx: MempoolTransaction }>());
const toggleSidePanel = createAction(type('Toggle Side Panel'));

export const MempoolActions = {
  init,
  close,
  getTxs,
  getTxsSuccess,
  changeFilters,
  setActiveTx,
  toggleSidePanel,
};
