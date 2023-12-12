import { isDesktop, lastItem, sort, SortDirection, TableSort, toggleItem } from '@openmina/shared';
import { NodesLiveState } from '@nodes/live/nodes-live.state';
import {
  NODES_LIVE_CLOSE,
  NODES_LIVE_GET_NODES_SUCCESS,
  NODES_LIVE_SET_ACTIVE_NODE,
  NODES_LIVE_SORT_EVENTS,
  NODES_LIVE_TOGGLE_FILTER,
  NODES_LIVE_TOGGLE_SIDE_PANEL,
  NodesLiveActions,
} from '@nodes/live/nodes-live.actions';
import { NodesLiveBlockEvent } from '@shared/types/nodes/live/nodes-live-block-event.type';
import { NodesOverviewNodeBlockStatus } from '@shared/types/nodes/dashboard/nodes-overview-block.type';

const initialState: NodesLiveState = {
  nodes: [],
  activeNode: undefined,
  openSidePanel: isDesktop(),
  sort: {
    sortBy: 'timestamp',
    sortDirection: SortDirection.DSC,
  },
  filteredEvents: [],
  filters: [],
};

export function reducer(state: NodesLiveState = initialState, action: NodesLiveActions): NodesLiveState {
  switch (action.type) {

    case NODES_LIVE_GET_NODES_SUCCESS: {
      let activeNode = state.activeNode ? action.payload.find(node => node.bestTip === state.activeNode.bestTip) : lastItem(action.payload);
      return {
        ...state,
        nodes: action.payload,
        activeNode,
        filteredEvents: filterEvents(sortEvents(activeNode?.events || [], state.sort), state.filters),
      };
    }

    case NODES_LIVE_SET_ACTIVE_NODE: {
      let activeNode = state.nodes.find(node => node.bestTip === action.payload.hash);
      if (!activeNode) {
        return state;
      }
      return {
        ...state,
        activeNode,
        filteredEvents: filterEvents(sortEvents(activeNode.events, state.sort), state.filters),
      };
    }

    case NODES_LIVE_SORT_EVENTS: {
      return {
        ...state,
        sort: action.payload,
        filteredEvents: filterEvents(sortEvents(state.activeNode.events, action.payload), state.filters),
      };
    }

    case NODES_LIVE_TOGGLE_SIDE_PANEL: {
      return {
        ...state,
        openSidePanel: !state.openSidePanel,
        activeNode: state.openSidePanel ? undefined : state.activeNode,
      };
    }

    case NODES_LIVE_TOGGLE_FILTER: {
      const filters = toggleItem(state.filters, action.payload);
      return {
        ...state,
        filters,
        filteredEvents: filterEvents(sortEvents(state.activeNode.events, state.sort), filters),
      }
    }

    case NODES_LIVE_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function sortEvents(events: NodesLiveBlockEvent[], tableSort: TableSort<NodesLiveBlockEvent>): NodesLiveBlockEvent[] {
  return sort<NodesLiveBlockEvent>(events, tableSort, ['message', 'status']);
}

function filterEvents(events: NodesLiveBlockEvent[], filters: string[]): NodesLiveBlockEvent[] {
  if (!filters.length) {
    return events;
  }
  if (filters.includes('best tip')) {
    events = events.filter(event => event.isBestTip);
  }

  if (filters.some(f => Object.values(NodesOverviewNodeBlockStatus).includes(f as NodesOverviewNodeBlockStatus))) {
    events = events.filter(event => filters.includes(event.message));
  }

  return events;
}
