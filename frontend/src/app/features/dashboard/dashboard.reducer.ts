import { DashboardState } from '@dashboard/dashboard.state';
import {
  DASHBOARD_CLOSE,
  DASHBOARD_GET_DATA_SUCCESS,
  DASHBOARD_PEERS_SORT,
  DashboardActions,
} from '@dashboard/dashboard.actions';
import { DashboardPeer, DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { sort, SortDirection, TableSort } from '@openmina/shared';

const initialState: DashboardState = {
  peers: [],
  peersStats: {
    connected: 0,
    connecting: 0,
    disconnected: 0,
  },
  peersSort: {
    sortBy: 'height',
    sortDirection: SortDirection.DSC,
  },
  nodes: [],
  rpcStats: {
    peerResponses: [],
    stakingLedger: null,
    nextLedger: null,
    rootLedger: null,
  },
  nodeBootstrappingPercentage: 0,
  appliedBlocks: 0,
  maxBlockHeightSeen: 0,
  berkeleyBlockHeight: 0,
  receivedBlocks: 0,
  receivedTxs: 0,
  receivedSnarks: 0,
};

export function dashboardReducer(state: DashboardState = initialState, action: DashboardActions): DashboardState {
  switch (action.type) {

    case DASHBOARD_PEERS_SORT: {
      return {
        ...state,
        peersSort: action.payload,
        peers: sortPeers(state.peers, action.payload),
      };
    }

    case DASHBOARD_GET_DATA_SUCCESS: {
      let peers = action.payload.peers.map(peer => ({
        ...peer,
        requests: action.payload.rpcStats.peerResponses.find(r => r.peerId === peer.peerId)?.requestsMade || 0,
      }));
      peers = sortPeers(peers, state.peersSort);
      return {
        ...state,
        peers,
        peersStats: {
          connected: peers.filter(peer => peer.status === DashboardPeerStatus.CONNECTED).length,
          connecting: peers.filter(peer => peer.status === DashboardPeerStatus.CONNECTING).length,
          disconnected: peers.filter(peer => peer.status === DashboardPeerStatus.DISCONNECTED).length,
        },
        nodes: action.payload.ledger,
        rpcStats: action.payload.rpcStats,
      };
    }

    case DASHBOARD_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function sortPeers(node: DashboardPeer[], tableSort: TableSort<DashboardPeer>): DashboardPeer[] {
  return sort<DashboardPeer>(node, tableSort, ['peerId', 'status', 'bestTip', 'address']);
}
