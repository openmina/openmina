import { NodesOverviewState } from '@nodes/overview/nodes-overview.state';
import {
  NODES_OVERVIEW_CLOSE,
  NODES_OVERVIEW_GET_NODE_STATUS_SUCCESS,
  NODES_OVERVIEW_SET_ACTIVE_NODE,
  NODES_OVERVIEW_SORT_NODES,
  NodesOverviewActions,
} from '@nodes/overview/nodes-overview.actions';
import { sort, SortDirection, TableSort } from '@openmina/shared';
import { NodesOverviewNode, NodesOverviewNodeKindType } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { CONFIG } from '@shared/constants/config';

const initialState: NodesOverviewState = {
  nodes: CONFIG.configs.map(node => ({
    name: node.name,
    kind: NodesOverviewNodeKindType.PENDING,
    blocks: [],
  } as NodesOverviewNode)),
  activeNode: undefined,
  sort: {
    sortBy: 'kind',
    sortDirection: SortDirection.DSC,
  },
};

export function reducer(state: NodesOverviewState = initialState, action: NodesOverviewActions): NodesOverviewState {
  switch (action.type) {

    case NODES_OVERVIEW_GET_NODE_STATUS_SUCCESS: {
      const currentNode = action.payload;
      let nodes = [...state.nodes];

      if (currentNode) {
        const nodeIndex = nodes.findIndex(node => node.name === currentNode.name);
        if (nodeIndex !== -1) {
          nodes[nodeIndex] = currentNode;
        } else {
          nodes = [...nodes, currentNode];
        }
      }

      return {
        ...state,
        nodes: sortNodes(nodes, state.sort),
        activeNode: state.activeNode ? nodes.find(node => node.name === state.activeNode.name) : undefined,
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
  return sort<NodesOverviewNode>(node, tableSort, ['kind', 'name', 'bestTip']);
}
