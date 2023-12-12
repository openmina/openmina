import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { SnarksWorkPoolState } from '@snarks/work-pool/snarks-work-pool.state';
import { ScanStateState } from '@snarks/scan-state/scan-state.state';

export interface SnarksState {
  workPool: SnarksWorkPoolState;
  scanState: ScanStateState;
}

const select = <T>(selector: (state: SnarksState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectSnarksState,
  selector,
);

export const selectSnarksState = createFeatureSelector<SnarksState>('snarks');
export const selectSnarksWorkPoolState = select((state: SnarksState): SnarksWorkPoolState => state.workPool);
export const selectSnarksScanStateState = select((state: SnarksState): ScanStateState => state.scanState);
