import { FeatureAction, TableSort } from '@openmina/shared';
import { DashboardPeer } from '@shared/types/dashboard/dashboard.peer';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { DashboardRpcStats } from '@shared/types/dashboard/dashboard-rpc-stats.type';

enum DashboardActionTypes {
  DASHBOARD_INIT = 'DASHBOARD_INIT',
  DASHBOARD_GET_DATA = 'DASHBOARD_GET_DATA',
  DASHBOARD_GET_DATA_SUCCESS = 'DASHBOARD_GET_DATA_SUCCESS',
  DASHBOARD_GET_PEERS = 'DASHBOARD_GET_PEERS',
  DASHBOARD_GET_PEERS_SUCCESS = 'DASHBOARD_GET_PEERS_SUCCESS',
  DASHBOARD_PEERS_SORT = 'DASHBOARD_PEERS_SORT',
  DASHBOARD_CLOSE = 'DASHBOARD_CLOSE',
}

export const DASHBOARD_INIT = DashboardActionTypes.DASHBOARD_INIT;
export const DASHBOARD_GET_DATA = DashboardActionTypes.DASHBOARD_GET_DATA;
export const DASHBOARD_GET_DATA_SUCCESS = DashboardActionTypes.DASHBOARD_GET_DATA_SUCCESS;
export const DASHBOARD_GET_PEERS = DashboardActionTypes.DASHBOARD_GET_PEERS;
export const DASHBOARD_GET_PEERS_SUCCESS = DashboardActionTypes.DASHBOARD_GET_PEERS_SUCCESS;
export const DASHBOARD_PEERS_SORT = DashboardActionTypes.DASHBOARD_PEERS_SORT;
export const DASHBOARD_CLOSE = DashboardActionTypes.DASHBOARD_CLOSE;

export interface DashboardAction extends FeatureAction<DashboardActionTypes> {
  readonly type: DashboardActionTypes;
}

export class DashboardInit implements DashboardAction {
  readonly type = DASHBOARD_INIT;
}

export class DashboardGetData implements DashboardAction {
  readonly type = DASHBOARD_GET_DATA;

  constructor(public payload?: { force: boolean }) {}
}

export class DashboardGetDataSuccess implements DashboardAction {
  readonly type = DASHBOARD_GET_DATA_SUCCESS;

  constructor(public payload: { peers: DashboardPeer[], ledger: NodesOverviewNode[], rpcStats: DashboardRpcStats }) { }
}

export class DashboardGetPeers implements DashboardAction {
  readonly type = DASHBOARD_GET_PEERS;
}

export class DashboardGetPeersSuccess implements DashboardAction {
  readonly type = DASHBOARD_GET_PEERS_SUCCESS;

  constructor(public payload: DashboardPeer[]) { }
}

export class DashboardPeersSort implements DashboardAction {
  readonly type = DASHBOARD_PEERS_SORT;

  constructor(public payload: TableSort<DashboardPeer>) { }
}

export class DashboardClose implements DashboardAction {
  readonly type = DASHBOARD_CLOSE;
}

export type DashboardActions =
  | DashboardInit
  | DashboardGetData
  | DashboardGetDataSuccess
  | DashboardGetPeers
  | DashboardGetPeersSuccess
  | DashboardPeersSort
  | DashboardClose
  ;
