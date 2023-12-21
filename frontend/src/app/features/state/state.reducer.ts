import { ActionReducer, combineReducers } from '@ngrx/store';

import * as fromActions from '@state/actions/state-actions.reducer';
import { StateActionsAction, StateActionsActions } from '@state/actions/state-actions.actions';

import { StateState } from '@state/state.state';

export type StateActions =
  & StateActionsActions
  ;
export type StateAction =
  & StateActionsAction
  ;

export const stateReducer: ActionReducer<StateState, StateActions> = combineReducers<StateState, StateActions>({
  actions: fromActions.reducer,
});
