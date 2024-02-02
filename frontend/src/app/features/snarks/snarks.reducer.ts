import { ActionReducer, combineReducers } from '@ngrx/store';

import { SnarksState } from '@snarks/snarks.state';

import * as fromWorkPool from '@snarks/work-pool/snarks-work-pool.reducer';
import { SnarksWorkPoolAction, SnarksWorkPoolActions } from '@snarks/work-pool/snarks-work-pool.actions';

import * as fromScanState from '@snarks/scan-state/scan-state.reducer';
import { ScanStateAction, ScanStateActions } from '@snarks/scan-state/scan-state.actions';

export type SnarksActions =
  & SnarksWorkPoolActions
  & ScanStateActions
  ;
export type SnarksAction =
  & SnarksWorkPoolAction
  & ScanStateAction
  ;

export const snarksReducer: ActionReducer<SnarksState, SnarksActions> = combineReducers<SnarksState, SnarksActions>({
  workPool: fromWorkPool.reducer,
  scanState: fromScanState.reducer,
});
