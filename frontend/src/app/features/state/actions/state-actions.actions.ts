import { FeatureAction, TableSort } from '@openmina/shared';
import { StateActionGroup } from '@shared/types/state/actions/state-action-group.type';
import { StateActionsStats } from '@shared/types/state/actions/state-actions-stats.type';

enum StateActionsActionTypes {
  STATE_ACTIONS_GET_EARLIEST_SLOT = 'STATE_ACTIONS_GET_EARLIEST_SLOT',
  STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS = 'STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS',
  STATE_ACTIONS_GET_ACTIONS = 'STATE_ACTIONS_GET_ACTIONS',
  STATE_ACTIONS_GET_ACTIONS_SUCCESS = 'STATE_ACTIONS_GET_ACTIONS_SUCCESS',
  STATE_ACTIONS_CLOSE = 'STATE_ACTIONS_CLOSE',
  STATE_ACTIONS_TOGGLE_SIDE_PANEL = 'STATE_ACTIONS_TOGGLE_SIDE_PANEL',
  STATE_ACTIONS_SORT = 'STATE_ACTIONS_SORT',
  STATE_ACTIONS_SEARCH = 'STATE_ACTIONS_SEARCH',
}

export const STATE_ACTIONS_GET_EARLIEST_SLOT = StateActionsActionTypes.STATE_ACTIONS_GET_EARLIEST_SLOT;
export const STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS = StateActionsActionTypes.STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS;
export const STATE_ACTIONS_GET_ACTIONS = StateActionsActionTypes.STATE_ACTIONS_GET_ACTIONS;
export const STATE_ACTIONS_GET_ACTIONS_SUCCESS = StateActionsActionTypes.STATE_ACTIONS_GET_ACTIONS_SUCCESS;
export const STATE_ACTIONS_CLOSE = StateActionsActionTypes.STATE_ACTIONS_CLOSE;
export const STATE_ACTIONS_TOGGLE_SIDE_PANEL = StateActionsActionTypes.STATE_ACTIONS_TOGGLE_SIDE_PANEL;
export const STATE_ACTIONS_SORT = StateActionsActionTypes.STATE_ACTIONS_SORT;
export const STATE_ACTIONS_SEARCH = StateActionsActionTypes.STATE_ACTIONS_SEARCH;

export interface StateActionsAction extends FeatureAction<StateActionsActionTypes> {
  readonly type: StateActionsActionTypes;
}

export class StateActionsGetEarliestSlot implements StateActionsAction {
  readonly type = STATE_ACTIONS_GET_EARLIEST_SLOT;
}

export class StateActionsGetEarliestSlotSuccess implements StateActionsAction {
  readonly type = STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS;

  constructor(public payload: number) { }
}

export class StateActionsGetActions implements StateActionsAction {
  readonly type = STATE_ACTIONS_GET_ACTIONS;

  constructor(public payload: { slot: number }) { }
}

export class StateActionsGetActionsSuccess implements StateActionsAction {
  readonly type = STATE_ACTIONS_GET_ACTIONS_SUCCESS;

  constructor(public payload: [StateActionsStats, StateActionGroup[]]) { }
}

export class StateActionsClose implements StateActionsAction {
  readonly type = STATE_ACTIONS_CLOSE;
}

export class StateActionsToggleSidePanel implements StateActionsAction {
  readonly type = STATE_ACTIONS_TOGGLE_SIDE_PANEL;
}

export class StateActionsSort implements StateActionsAction {
  readonly type = STATE_ACTIONS_SORT;

  constructor(public payload: TableSort<StateActionGroup>) { }
}

export class StateActionsSearch implements StateActionsAction {
  readonly type = STATE_ACTIONS_SEARCH;

  constructor(public payload: string) { }
}

export type StateActionsActions =
  | StateActionsGetEarliestSlot
  | StateActionsGetEarliestSlotSuccess
  | StateActionsGetActions
  | StateActionsGetActionsSuccess
  | StateActionsClose
  | StateActionsToggleSidePanel
  | StateActionsSort
  | StateActionsSearch
  ;
