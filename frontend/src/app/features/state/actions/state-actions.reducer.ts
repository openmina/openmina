import { StateActionsState } from '@state/actions/state-actions.state';
import {
  STATE_ACTIONS_CLOSE,
  STATE_ACTIONS_GET_ACTIONS,
  STATE_ACTIONS_GET_ACTIONS_SUCCESS,
  STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS,
  STATE_ACTIONS_SEARCH,
  STATE_ACTIONS_SORT,
  STATE_ACTIONS_TOGGLE_SIDE_PANEL,
  StateActionsActions,
} from '@state/actions/state-actions.actions';
import { isMobile, sort, SortDirection, TableSort } from '@openmina/shared';
import { StateActionGroup } from '@shared/types/state/actions/state-action-group.type';
import { StateActionsStats } from '@shared/types/state/actions/state-actions-stats.type';

const initialState: StateActionsState = {
  groups: [],
  filteredGroups: [],
  stats: {} as StateActionsStats,
  openSidePanel: !isMobile(),
  activeSlot: undefined,
  earliestSlot: undefined,
  activeSearch: '',
  currentSort: {
    sortBy: 'totalTime',
    sortDirection: SortDirection.DSC,
  },
};

export function reducer(state: StateActionsState = initialState, action: StateActionsActions): StateActionsState {
  switch (action.type) {

    case STATE_ACTIONS_GET_ACTIONS: {
      return {
        ...state,
        activeSlot: action.payload.slot,
      };
    }

    case STATE_ACTIONS_GET_ACTIONS_SUCCESS: {
      const groups = sortGroups(action.payload[1], state.currentSort);
      return {
        ...state,
        groups,
        filteredGroups: searchActionsInGroups(state.activeSearch, groups),
        stats: action.payload[0],
      };
    }

    case STATE_ACTIONS_TOGGLE_SIDE_PANEL: {
      return {
        ...state,
        openSidePanel: !state.openSidePanel,
      };
    }

    case STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS: {
      return {
        ...state,
        earliestSlot: action.payload,
      };
    }

    case STATE_ACTIONS_SORT: {
      return {
        ...state,
        currentSort: action.payload,
        filteredGroups: sortGroups(state.filteredGroups, action.payload),
      };
    }

    case STATE_ACTIONS_SEARCH: {
      const groups = searchActionsInGroups(action.payload, state.groups);
      return {
        ...state,
        activeSearch: action.payload,
        filteredGroups: sortGroups(groups, state.currentSort),
      };
    }

    case STATE_ACTIONS_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function searchActionsInGroups(toSearch: string, groups: StateActionGroup[]): StateActionGroup[] {
  return !toSearch
    ? groups.map(group => ({ ...group, display: true }))
    : groups.map(group => {
      return {
        ...group,
        actions: group.actions.map(action => {
          if (action.fullTitle.toLowerCase().includes(toSearch.toLowerCase())) {
            return { ...action, display: true };
          }
          return { ...action, display: false };
        }),
      };
    }).map(group => {
      if (group.actions.some(action => action.display)) {
        return { ...group, display: true };
      }
      return { ...group, display: false };
    });
}

function sortGroups(blocks: StateActionGroup[], tableSort: TableSort<StateActionGroup>): StateActionGroup[] {
  return sort<StateActionGroup>(blocks, tableSort, []);
}
