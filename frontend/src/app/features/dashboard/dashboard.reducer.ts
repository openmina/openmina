import { DashboardState } from '@dashboard/dashboard.state';
import { DASHBOARD_CLOSE, DASHBOARD_GET_PEERS_SUCCESS, DASHBOARD_PEERS_SORT, DashboardActions } from '@dashboard/dashboard.actions';
import { DashboardPeer, DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { SortDirection, TableSort } from '@openmina/shared';

const initialState: DashboardState = {
  peers: [],
  peersStats: {
    connected: 0,
    connecting: 0,
    disconnected: 0,
  },
  peersSort: {
    sortBy: 'timestamp',
    sortDirection: SortDirection.DSC,
  },
  nodeBootstrappingPercentage: 0,
  appliedBlocks: 0,
  maxBlockHeightSeen: 0,
  berkeleyBlockHeight: 0,
  receivedBlocks: 0,
  receivedTxs: 0,
  receivedSnarks: 0,
};

export function reducer(state: DashboardState = initialState, action: DashboardActions): DashboardState {
  switch (action.type) {

    case DASHBOARD_GET_PEERS_SUCCESS: {
      const peers = sortPeers(action.payload, state.peersSort);
      return {
        ...state,
        peers,
        peersStats: {
          connected: peers.filter(peer => peer.status === DashboardPeerStatus.CONNECTED).length,
          connecting: peers.filter(peer => peer.status === DashboardPeerStatus.CONNECTING).length,
          disconnected: peers.filter(peer => peer.status === DashboardPeerStatus.DISCONNECTED).length,
        },
      };
    }

    case DASHBOARD_PEERS_SORT: {
      return {
        ...state,
        peersSort: action.payload,
        peers: sortPeers(state.peers, action.payload),
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

export function sort<T = any>(inpArray: T[], sort: TableSort<T>, strings: Array<keyof T>, sortNulls: boolean = false): T[] {
  const sortProperty = sort.sortBy;
  const isStringSorting = strings.includes(sortProperty);
  const array: T[] = [...inpArray];

  let toBeSorted: T[];
  let toNotBeSorted: T[] = [];
  if (sortNulls) {
    toBeSorted = array;
  } else {
    toBeSorted = isStringSorting ? array : array.filter(e => e[sortProperty] !== undefined && e[sortProperty] !== null);
    toNotBeSorted = isStringSorting ? [] : array.filter(e => e[sortProperty] === undefined || e[sortProperty] === null);
  }

  if (isStringSorting) {
    const stringSort = (o1: T, o2: T) => {
      const s2 = (o2[sortProperty] || '') as string;
      const s1 = (o1[sortProperty] || '') as string;
      return sort.sortDirection === SortDirection.DSC
        ? (s2).localeCompare(s1)
        : s1.localeCompare(s2);
    };
    toBeSorted.sort(stringSort);
  } else {
    const numberSort = (o1: T, o2: T): number => {
      const o2Sort = (o2[sortProperty] ?? Number.MAX_VALUE) as number;
      const o1Sort = (o1[sortProperty] ?? Number.MAX_VALUE) as number;
      return sort.sortDirection === SortDirection.DSC
        ? o2Sort - o1Sort
        : o1Sort - o2Sort;
    };
    toBeSorted.sort(numberSort);
  }

  return [...toBeSorted, ...toNotBeSorted];
}
