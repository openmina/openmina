import { NetworkBootstrapStatsState } from '@network/bootstrap-stats/network-bootstrap-stats.state';
import {
  NETWORK_BOOTSTRAP_STATS_CLOSE,
  NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS,
  NETWORK_BOOTSTRAP_STATS_SET_ACTIVE_BOOTSTRAP_REQUEST,
  NETWORK_BOOTSTRAP_STATS_SORT,
  NetworkBootstrapStatsActions,
} from '@network/bootstrap-stats/network-bootstrap-stats.actions';
import { sort, SortDirection, TableSort } from '@openmina/shared';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';

const initialState: NetworkBootstrapStatsState = {
  boostrapStats: [],
  activeBootstrapRequest: undefined,
  sort: {
    sortBy: 'start',
    sortDirection: SortDirection.ASC,
  },
};

export function networkBootstrapStatsReducer(state: NetworkBootstrapStatsState = initialState, action: NetworkBootstrapStatsActions): NetworkBootstrapStatsState {
  switch (action.type) {
    case NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS: {
      return {
        ...state,
        boostrapStats: sortRequests(action.payload, state.sort),
      };
    }

    case NETWORK_BOOTSTRAP_STATS_SET_ACTIVE_BOOTSTRAP_REQUEST: {
      return {
        ...state,
        activeBootstrapRequest: action.payload,
      };
    }

    case NETWORK_BOOTSTRAP_STATS_SORT: {
      return {
        ...state,
        sort: action.payload,
        boostrapStats: sortRequests(state.boostrapStats, action.payload),
      };
    }

    case NETWORK_BOOTSTRAP_STATS_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function sortRequests(requests: NetworkBootstrapStatsRequest[], tableSort: TableSort<NetworkBootstrapStatsRequest>): NetworkBootstrapStatsRequest[] {
  return sort<NetworkBootstrapStatsRequest>(requests, tableSort, ['type', 'address', 'peerId']);
}
