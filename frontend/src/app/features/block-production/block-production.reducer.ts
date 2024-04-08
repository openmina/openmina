import { ActionReducer, combineReducers } from '@ngrx/store';
import { blockProductionOverviewReducer } from '@block-production/overview/block-production-overview.reducer';
import {
  BlockProductionOverviewAction,
  BlockProductionOverviewActions,
} from '@block-production/overview/block-production-overview.actions';
import { BlockProductionState } from '@block-production/block-production.state';

export type BlockProductionActions =
  & BlockProductionOverviewActions
  ;

export type BlockProductionAction =
  & BlockProductionOverviewAction
  ;

export const blockProductionReducer: ActionReducer<BlockProductionState, BlockProductionActions> = combineReducers<BlockProductionState, BlockProductionActions>({
  overview: blockProductionOverviewReducer,
});
