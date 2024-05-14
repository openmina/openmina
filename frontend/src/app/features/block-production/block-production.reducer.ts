import { ActionReducer, combineReducers } from '@ngrx/store';
import { blockProductionOverviewReducer } from '@block-production/overview/block-production-overview.reducer';
import { BlockProductionState } from '@block-production/block-production.state';
import { BLOCK_PRODUCTION_OVERVIEW_KEY } from '@block-production/overview/block-production-overview.actions';

export const blockProductionReducer: ActionReducer<BlockProductionState, any> = combineReducers<BlockProductionState, any>({
  [BLOCK_PRODUCTION_OVERVIEW_KEY]: blockProductionOverviewReducer,
});
