import { FeatureAction, TableSort } from '@openmina/shared';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';

enum NetworkBootstrapStatsActionTypes {
  NETWORK_BOOTSTRAP_STATS_INIT = 'NETWORK_BOOTSTRAP_STATS_INIT',
  NETWORK_BOOTSTRAP_STATS_CLOSE = 'NETWORK_BOOTSTRAP_STATS_CLOSE',
  NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS = 'NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS',
  NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS = 'NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS',
  NETWORK_BOOTSTRAP_STATS_SET_ACTIVE_BOOTSTRAP_REQUEST = 'NETWORK_BOOTSTRAP_STATS_SET_ACTIVE_BOOTSTRAP_REQUEST',
  NETWORK_BOOTSTRAP_STATS_SORT = 'NETWORK_BOOTSTRAP_STATS_SORT',
}

export const NETWORK_BOOTSTRAP_STATS_INIT = NetworkBootstrapStatsActionTypes.NETWORK_BOOTSTRAP_STATS_INIT;
export const NETWORK_BOOTSTRAP_STATS_CLOSE = NetworkBootstrapStatsActionTypes.NETWORK_BOOTSTRAP_STATS_CLOSE;
export const NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS = NetworkBootstrapStatsActionTypes.NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS;
export const NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS = NetworkBootstrapStatsActionTypes.NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS;
export const NETWORK_BOOTSTRAP_STATS_SET_ACTIVE_BOOTSTRAP_REQUEST = NetworkBootstrapStatsActionTypes.NETWORK_BOOTSTRAP_STATS_SET_ACTIVE_BOOTSTRAP_REQUEST;
export const NETWORK_BOOTSTRAP_STATS_SORT = NetworkBootstrapStatsActionTypes.NETWORK_BOOTSTRAP_STATS_SORT;

export interface NetworkBootstrapStatsAction extends FeatureAction<NetworkBootstrapStatsActionTypes> {
  readonly type: NetworkBootstrapStatsActionTypes;
}

export class NetworkBootstrapStatsInit implements NetworkBootstrapStatsAction {
  readonly type = NETWORK_BOOTSTRAP_STATS_INIT;
}

export class NetworkBootstrapStatsClose implements NetworkBootstrapStatsAction {
  readonly type = NETWORK_BOOTSTRAP_STATS_CLOSE;
}

export class NetworkBootstrapStatsGetBootstrapStats implements NetworkBootstrapStatsAction {
  readonly type = NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS;
}

export class NetworkBootstrapStatsGetBootstrapStatsSuccess implements NetworkBootstrapStatsAction {
  readonly type = NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS;

  constructor(public payload: NetworkBootstrapStatsRequest[]) { }
}

export class NetworkBootstrapStatsSort implements NetworkBootstrapStatsAction {
  readonly type = NETWORK_BOOTSTRAP_STATS_SORT;

  constructor(public payload: TableSort<NetworkBootstrapStatsRequest>) { }
}

export class NetworkBootstrapStatsSetActiveBootstrapRequest implements NetworkBootstrapStatsAction {
  readonly type = NETWORK_BOOTSTRAP_STATS_SET_ACTIVE_BOOTSTRAP_REQUEST;

  constructor(public payload: NetworkBootstrapStatsRequest) { }
}

export type NetworkBootstrapStatsActions =
  | NetworkBootstrapStatsInit
  | NetworkBootstrapStatsClose
  | NetworkBootstrapStatsGetBootstrapStats
  | NetworkBootstrapStatsGetBootstrapStatsSuccess
  | NetworkBootstrapStatsSort
  | NetworkBootstrapStatsSetActiveBootstrapRequest
  ;
