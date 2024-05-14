import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import {
  BlockProductionOverviewState,
} from '@block-production/overview/block-production-overview.state';
import { BLOCK_PRODUCTION_OVERVIEW_KEY } from '@block-production/overview/block-production-overview.actions';

const select = <T>(selector: (state: BlockProductionState) => T): MemoizedSelector<MinaState, T> => createSelector(
  createFeatureSelector<BlockProductionState>('blockProduction'),
  selector,
);

const overview = select((state: BlockProductionState): BlockProductionOverviewState => state.overview);
export const BlockProductionSelectors = {
  overview,
};

export interface BlockProductionState {
  [BLOCK_PRODUCTION_OVERVIEW_KEY]: BlockProductionOverviewState;
}
