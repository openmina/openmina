import { isMobile, noMillisFormat, sort, SortDirection, TableSort, toReadableDate } from '@openmina/shared';
import { DashboardSplitsState } from '@network/splits/dashboard-splits.state';
import {
  DASHBOARD_SPLITS_CLOSE,
  DASHBOARD_SPLITS_GET_SPLITS,
  DASHBOARD_SPLITS_GET_SPLITS_SUCCESS,
  DASHBOARD_SPLITS_MERGE_NODES,
  DASHBOARD_SPLITS_SET_ACTIVE_PEER,
  DASHBOARD_SPLITS_SORT_PEERS,
  DASHBOARD_SPLITS_SPLIT_NODES,
  DASHBOARD_SPLITS_TOGGLE_SIDE_PANEL,
  DashboardSplitsActions,
} from '@network/splits/dashboard-splits.actions';
import { DashboardSplitsLink } from '@shared/types/network/splits/dashboard-splits-link.type';
import { DashboardNodeCount } from '@shared/types/network/splits/dashboard-node-count.type';
import { DashboardSplitsPeer } from '@shared/types/network/splits/dashboard-splits-peer.type';
import { DashboardSplitsSet } from '@shared/types/network/splits/dashboard-splits-set.type';

const initialState: DashboardSplitsState = {
  peers: [],
  links: [],
  sets: [],
  activePeer: undefined,
  networkSplitsDetails: undefined,
  networkMergeDetails: undefined,
  nodeStats: undefined,
  fetching: true,
  sort: {
    sortBy: 'outgoingConnections',
    sortDirection: SortDirection.DSC,
  },
  openSidePanel: !isMobile(),
};

export function topologyReducer(state: DashboardSplitsState = initialState, action: DashboardSplitsActions): DashboardSplitsState {
  switch (action.type) {

    case DASHBOARD_SPLITS_GET_SPLITS: {
      return {
        ...state,
        activePeer: undefined,
        fetching: true,
      };
    }

    case DASHBOARD_SPLITS_GET_SPLITS_SUCCESS: {
      const peers = action.payload.peers.map((p) => ({
        ...p,
        radius: getRadius(p.address, action.payload.links),
        outgoingConnections: action.payload.links.filter(l => l.source === p.address).length,
        incomingConnections: action.payload.links.filter(l => l.target === p.address).length,
      }));
      const sets = splitThePeers(peers, action.payload.links);
      return {
        ...state,
        peers,
        sets: sets.map(set => ({ ...set, peers: sortPeers(set.peers, state.sort) })),
        links: action.payload.links,
        nodeStats: getNodeCount(peers.map(p => ({ url: !p.node ? 'node' : p.node.toLowerCase() }))),
        fetching: false,
      };
    }

    case DASHBOARD_SPLITS_SET_ACTIVE_PEER: {
      return {
        ...state,
        activePeer: action.payload,
        openSidePanel: true,
      };
    }

    case DASHBOARD_SPLITS_SPLIT_NODES: {
      return {
        ...state,
        networkSplitsDetails: 'Last split: ' + toReadableDate(Date.now(), noMillisFormat),
      };
    }

    case DASHBOARD_SPLITS_MERGE_NODES: {
      return {
        ...state,
        networkMergeDetails: 'Last merge: ' + toReadableDate(Date.now(), noMillisFormat),
      };
    }

    case DASHBOARD_SPLITS_SORT_PEERS: {
      return {
        ...state,
        sort: action.payload,
        sets: state.sets.map(set => ({
          ...set,
          peers: sortPeers(set.peers, action.payload),
        })),
      };
    }

    case DASHBOARD_SPLITS_TOGGLE_SIDE_PANEL: {
      return {
        ...state,
        openSidePanel: !state.openSidePanel,
      };
    }

    case DASHBOARD_SPLITS_CLOSE:
      return initialState;

    default:
      return state;
  }
}


function getRadius(address: string, links: DashboardSplitsLink[]): number {
  const occurrence = links.filter(link => link.source === address || link.target === address).length;
  if (occurrence < 6) {
    return 4;
  } else if (occurrence < 8) {
    return 6;
  } else if (occurrence < 10) {
    return 8;
  } else if (occurrence < 12) {
    return 10;
  } else if (occurrence < 14) {
    return 12;
  } else {
    return 14;
  }
}

function getNodeCount<T extends { url: string }>(nodes: T[]): DashboardNodeCount {
  return {
    nodes: (nodes.filter(node => node.url.includes('node')).map(n => n.url)).length,
    producers: (nodes.filter(node => node.url.includes('prod')).map(n => n.url)).length,
    snarkers: (nodes.filter(node => node.url.includes('snarker')).map(n => n.url)).length,
    seeders: (nodes.filter(node => node.url.includes('seed')).map(n => n.url)).length,
    transactionGenerators: (nodes.filter(node => node.url.includes('transaction-generator')).map(n => n.url)).length,
  };
}

function splitThePeers(peers: DashboardSplitsPeer[], links: DashboardSplitsLink[]): DashboardSplitsSet[] {
  const sets: DashboardSplitsSet[] = [];
  const visited = new Set<string>();

  for (const peer of peers) {
    if (!visited.has(peer.address)) {
      const peersForCurrentSet = new Set<DashboardSplitsPeer>();
      const queue = [peer];

      while (queue.length > 0) {
        const currentPeer = queue.shift()!;

        if (!peersForCurrentSet.has(currentPeer)) {
          peersForCurrentSet.add(currentPeer);
          visited.add(currentPeer.address);

          for (const link of links) {
            if (link.source === currentPeer.address && !visited.has(link.target)) {
              queue.push(peers.find(p => p.address === link.target));
            } else if (link.target === currentPeer.address && !visited.has(link.source)) {
              queue.push(peers.find(p => p.address === link.source));
            }
          }
        }
      }

      sets.push({
        peers: sortPeers(Array.from(peersForCurrentSet), {
          sortBy: 'outgoingConnections',
          sortDirection: SortDirection.DSC,
        }),
      });
    }
  }

  return sets;
}

function sortPeers(messages: DashboardSplitsPeer[], tableSort: TableSort<DashboardSplitsPeer>): DashboardSplitsPeer[] {
  return sort<DashboardSplitsPeer>(messages, tableSort, ['address', 'peerId', 'node'], true);
}
