import { FeatureAction } from '@openmina/shared';
import { MinaState } from '@app/app.setup';
import { APP_INIT, APP_INIT_SUCCESS } from '@app/app.actions';
import {
  STATE_ACTIONS_CLOSE,
  STATE_ACTIONS_GET_ACTIONS,
  STATE_ACTIONS_GET_ACTIONS_SUCCESS,
  STATE_ACTIONS_GET_EARLIEST_SLOT,
  STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS,
} from '@state/actions/state-actions.actions';
import {
  NODES_OVERVIEW_CLOSE,
  NODES_OVERVIEW_GET_NODES_SUCCESS,
  NODES_OVERVIEW_INIT,
} from '@nodes/overview/nodes-overview.actions';
import {
  NODES_BOOTSTRAP_CLOSE,
  NODES_BOOTSTRAP_GET_NODES_SUCCESS,
  NODES_BOOTSTRAP_INIT,
} from '@nodes/bootstrap/nodes-bootstrap.actions';
import { NODES_LIVE_CLOSE, NODES_LIVE_GET_NODES_SUCCESS, NODES_LIVE_INIT } from '@nodes/live/nodes-live.actions';
import {
  SNARKS_WORK_POOL_CLOSE,
  SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL,
  SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS,
  SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS,
  SNARKS_WORK_POOL_INIT,
} from '@snarks/work-pool/snarks-work-pool.actions';
import { SCAN_STATE_CLOSE, SCAN_STATE_GET_BLOCK_SUCCESS, SCAN_STATE_INIT } from '@snarks/scan-state/scan-state.actions';
import { DASHBOARD_CLOSE, DASHBOARD_GET_DATA_SUCCESS, DASHBOARD_INIT } from '@dashboard/dashboard.actions';
import {
  MEMORY_RESOURCES_CLOSE,
  MEMORY_RESOURCES_GET,
  MEMORY_RESOURCES_GET_SUCCESS,
} from '@resources/memory/memory-resources.actions';
import {
  NETWORK_NODE_DHT_CLOSE,
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS,
  NETWORK_NODE_DHT_INIT,
} from '@network/node-dht/network-node-dht.actions';
import {
  NETWORK_BOOTSTRAP_STATS_CLOSE,
  NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS,
  NETWORK_BOOTSTRAP_STATS_INIT,
} from '@network/bootstrap-stats/network-bootstrap-stats.actions';

export type LoadingState = string[];

const initialState: LoadingState = [];

export function loadingReducer(state: LoadingState = initialState, action: FeatureAction<any>): LoadingState {
  switch (action.type) {
    /* ------------ ADD ------------ */
    case APP_INIT:

    case DASHBOARD_INIT:

    case STATE_ACTIONS_GET_EARLIEST_SLOT:
    case STATE_ACTIONS_GET_ACTIONS:

    case NODES_OVERVIEW_INIT:
    case NODES_BOOTSTRAP_INIT:
    case NODES_LIVE_INIT:

    case SNARKS_WORK_POOL_INIT:
    case SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL:

    case SCAN_STATE_INIT:

    case MEMORY_RESOURCES_GET:

    case NETWORK_NODE_DHT_INIT:
    case NETWORK_BOOTSTRAP_STATS_INIT:
      return add(state, action);

    /* ------------ REMOVE ------------ */
    case APP_INIT_SUCCESS:
      return remove(state, APP_INIT);

    case DASHBOARD_GET_DATA_SUCCESS:
      return remove(state, DASHBOARD_INIT);
    case DASHBOARD_CLOSE:
      return remove(state, [DASHBOARD_INIT]);

    case STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS:
      return remove(state, STATE_ACTIONS_GET_EARLIEST_SLOT);
    case STATE_ACTIONS_GET_ACTIONS_SUCCESS:
      return remove(state, STATE_ACTIONS_GET_ACTIONS);
    case STATE_ACTIONS_CLOSE:
      return remove(state, [STATE_ACTIONS_GET_EARLIEST_SLOT, STATE_ACTIONS_GET_ACTIONS]);

    case NODES_OVERVIEW_GET_NODES_SUCCESS:
      return remove(state, NODES_OVERVIEW_INIT);
    case NODES_OVERVIEW_CLOSE:
      return remove(state, [NODES_OVERVIEW_INIT]);
    case NODES_BOOTSTRAP_GET_NODES_SUCCESS:
      return remove(state, NODES_BOOTSTRAP_INIT);
    case NODES_BOOTSTRAP_CLOSE:
      return remove(state, [NODES_BOOTSTRAP_INIT]);
    case NODES_LIVE_GET_NODES_SUCCESS:
      return remove(state, NODES_LIVE_INIT);
    case NODES_LIVE_CLOSE:
      return remove(state, [NODES_LIVE_INIT]);

    case SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS:
      return remove(state, SNARKS_WORK_POOL_INIT);
    case SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS:
      return remove(state, SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL);
    case SNARKS_WORK_POOL_CLOSE:
      return remove(state, [SNARKS_WORK_POOL_INIT, SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL]);

    case SCAN_STATE_GET_BLOCK_SUCCESS:
      return remove(state, SCAN_STATE_INIT);
    case SCAN_STATE_CLOSE:
      return remove(state, [SCAN_STATE_INIT]);

    case MEMORY_RESOURCES_GET_SUCCESS:
      return remove(state, MEMORY_RESOURCES_GET);
    case MEMORY_RESOURCES_CLOSE:
      return remove(state, [MEMORY_RESOURCES_GET]);

    case NETWORK_NODE_DHT_GET_PEERS_SUCCESS:
      return remove(state, NETWORK_NODE_DHT_INIT);
    case NETWORK_NODE_DHT_CLOSE:
      return remove(state, [NETWORK_NODE_DHT_INIT]);

    case NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS:
      return remove(state, NETWORK_BOOTSTRAP_STATS_INIT);
    case NETWORK_BOOTSTRAP_STATS_CLOSE:
      return remove(state, [NETWORK_BOOTSTRAP_STATS_INIT]);

    default:
      return state;
  }
}

function add(state: LoadingState, action: FeatureAction<any>): LoadingState {
  return [action.type, ...state];
}

function remove(state: LoadingState, type: string | string[]): LoadingState {
  if (Array.isArray(type)) {
    return state.filter(t => !type.includes(t));
  }
  return state.filter(t => t !== type);
}

export const selectLoadingStateLength = (state: MinaState): number => state.loading.length;
