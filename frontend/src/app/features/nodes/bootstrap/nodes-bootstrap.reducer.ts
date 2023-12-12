import { sort, SortDirection, TableSort } from '@openmina/shared';
import { NodesBootstrapState } from '@nodes/bootstrap/nodes-bootstrap.state';
import {
  NODES_BOOTSTRAP_CLOSE,
  NODES_BOOTSTRAP_GET_NODES_SUCCESS,
  NODES_BOOTSTRAP_SET_ACTIVE_BLOCK,
  NODES_BOOTSTRAP_SORT_NODES,
  NODES_BOOTSTRAP_TOGGLE_SIDE_PANEL,
  NodesBootstrapActions,
} from '@nodes/bootstrap/nodes-bootstrap.actions';
import { NodesBootstrapNode } from '@shared/types/nodes/bootstrap/nodes-bootstrap-node.type';
import { isDesktop } from '@openmina/shared';

const initialState: NodesBootstrapState = {
  nodes: [],
  activeNode: undefined,
  openSidePanel: isDesktop(),
  sort: {
    sortBy: 'index',
    sortDirection: SortDirection.ASC,
  },
};

export function reducer(state: NodesBootstrapState = initialState, action: NodesBootstrapActions): NodesBootstrapState {
  switch (action.type) {

    case NODES_BOOTSTRAP_GET_NODES_SUCCESS: {
      return {
        ...state,
        nodes: sortNodes(action.payload, state.sort),
        activeNode: state.activeNode ? action.payload.find(node => node.index === state.activeNode.index) : undefined,
      };
    }

    case NODES_BOOTSTRAP_SET_ACTIVE_BLOCK: {
      return {
        ...state,
        activeNode: action.payload,
        openSidePanel: action.payload ? true : state.openSidePanel,
      };
    }

    case NODES_BOOTSTRAP_SORT_NODES: {
      return {
        ...state,
        sort: action.payload,
        nodes: sortNodes(state.nodes, action.payload),
      };
    }

    case NODES_BOOTSTRAP_TOGGLE_SIDE_PANEL: {
      return {
        ...state,
        openSidePanel: !state.openSidePanel,
        activeNode: state.openSidePanel ? undefined : state.activeNode,
      };
    }

    case NODES_BOOTSTRAP_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function sortNodes(node: NodesBootstrapNode[], tableSort: TableSort<NodesBootstrapNode>): NodesBootstrapNode[] {
  return sort<NodesBootstrapNode>(node, tableSort, ['bestTip']);
}
