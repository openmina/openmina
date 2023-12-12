import { NodesOverviewState } from '@nodes/overview/nodes-overview.state';
import {
  NODES_OVERVIEW_CLOSE,
  NODES_OVERVIEW_GET_NODES_SUCCESS,
  NODES_OVERVIEW_SET_ACTIVE_NODE,
  NODES_OVERVIEW_SORT_NODES,
  NodesOverviewActions,
} from '@nodes/overview/nodes-overview.actions';
import { sort, SortDirection, TableSort } from '@openmina/shared';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';

const initialState: NodesOverviewState = {
  nodes: [],
  activeNode: undefined,
  sort: {
    sortBy: 'kind',
    sortDirection: SortDirection.DSC,
  },
};

export function reducer(state: NodesOverviewState = initialState, action: NodesOverviewActions): NodesOverviewState {
  switch (action.type) {

    case NODES_OVERVIEW_GET_NODES_SUCCESS: {
      return {
        ...state,
        nodes: sortNodes(action.payload, state.sort),
        activeNode: state.activeNode ? action.payload.find(node => node.name === state.activeNode.name) : undefined,
      };
    }

    case NODES_OVERVIEW_SORT_NODES: {
      return {
        ...state,
        sort: action.payload,
        nodes: sortNodes(state.nodes, action.payload),
      };
    }

    case NODES_OVERVIEW_SET_ACTIVE_NODE: {
      return {
        ...state,
        activeNode: action.payload,
      };
    }

    case NODES_OVERVIEW_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function sortNodes(node: NodesOverviewNode[], tableSort: TableSort<NodesOverviewNode>): NodesOverviewNode[] {
  return sort<NodesOverviewNode>(node, tableSort, ['kind', 'name', 'bestTip',]);
}
