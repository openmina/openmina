import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';

export interface BlockProductionState {
  overview: BlockProductionOverviewState;
}

const select = <T>(selector: (state: BlockProductionState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectBlockProductionState,
  selector,
);

export const selectBlockProductionState = createFeatureSelector<BlockProductionState>('blockProduction');
export const selectBlockProductionOverviewState = select((state: BlockProductionState): BlockProductionOverviewState => state.overview);
