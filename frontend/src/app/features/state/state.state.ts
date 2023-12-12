import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { StateActionsState } from '@state/actions/state-actions.state';

export interface StateState {
  actions: StateActionsState;
}

const select = <T>(selector: (state: StateState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectStateState,
  selector,
);

export const selectStateState = createFeatureSelector<StateState>('state');
export const selectStateActionsState = select((state: StateState): StateActionsState => state.actions);
