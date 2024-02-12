import { FeatureAction, TableSort } from '@openmina/shared';
import { DashboardSplitsPeer } from '@shared/types/network/splits/dashboard-splits-peer.type';
import { DashboardSplits } from '@shared/types/network/splits/dashboard-splits.type';

enum DashboardSplitsActionTypes {
  DASHBOARD_SPLITS_CLOSE = 'DASHBOARD_SPLITS_CLOSE',
  DASHBOARD_SPLITS_GET_SPLITS = 'DASHBOARD_SPLITS_GET_SPLITS',
  DASHBOARD_SPLITS_GET_SPLITS_SUCCESS = 'DASHBOARD_SPLITS_GET_SPLITS_SUCCESS',
  DASHBOARD_SPLITS_SET_ACTIVE_PEER = 'DASHBOARD_SPLITS_SET_ACTIVE_PEER',
  DASHBOARD_SPLITS_SPLIT_NODES = 'DASHBOARD_SPLITS_SPLIT_NODES',
  DASHBOARD_SPLITS_SPLIT_NODES_SUCCESS = 'DASHBOARD_SPLITS_SPLIT_NODES_SUCCESS',
  DASHBOARD_SPLITS_MERGE_NODES = 'DASHBOARD_SPLITS_MERGE_NODES',
  DASHBOARD_SPLITS_MERGE_NODES_SUCCESS = 'DASHBOARD_SPLITS_MERGE_NODES_SUCCESS',
  DASHBOARD_SPLITS_SORT_PEERS = 'DASHBOARD_SPLITS_SORT_PEERS',
  DASHBOARD_SPLITS_TOGGLE_SIDE_PANEL = 'DASHBOARD_SPLITS_TOGGLE_SIDE_PANEL',
}

export const DASHBOARD_SPLITS_CLOSE = DashboardSplitsActionTypes.DASHBOARD_SPLITS_CLOSE;
export const DASHBOARD_SPLITS_GET_SPLITS = DashboardSplitsActionTypes.DASHBOARD_SPLITS_GET_SPLITS;
export const DASHBOARD_SPLITS_GET_SPLITS_SUCCESS = DashboardSplitsActionTypes.DASHBOARD_SPLITS_GET_SPLITS_SUCCESS;
export const DASHBOARD_SPLITS_SET_ACTIVE_PEER = DashboardSplitsActionTypes.DASHBOARD_SPLITS_SET_ACTIVE_PEER;
export const DASHBOARD_SPLITS_SPLIT_NODES = DashboardSplitsActionTypes.DASHBOARD_SPLITS_SPLIT_NODES;
export const DASHBOARD_SPLITS_SPLIT_NODES_SUCCESS = DashboardSplitsActionTypes.DASHBOARD_SPLITS_SPLIT_NODES_SUCCESS;
export const DASHBOARD_SPLITS_MERGE_NODES = DashboardSplitsActionTypes.DASHBOARD_SPLITS_MERGE_NODES;
export const DASHBOARD_SPLITS_MERGE_NODES_SUCCESS = DashboardSplitsActionTypes.DASHBOARD_SPLITS_MERGE_NODES_SUCCESS;
export const DASHBOARD_SPLITS_SORT_PEERS = DashboardSplitsActionTypes.DASHBOARD_SPLITS_SORT_PEERS;
export const DASHBOARD_SPLITS_TOGGLE_SIDE_PANEL = DashboardSplitsActionTypes.DASHBOARD_SPLITS_TOGGLE_SIDE_PANEL;

export interface DashboardSplitsAction extends FeatureAction<DashboardSplitsActionTypes> {
  readonly type: DashboardSplitsActionTypes;
}

export class DashboardSplitsClose implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_CLOSE;
}

export class DashboardSplitsGetSplits implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_GET_SPLITS;
}

export class DashboardSplitsGetSplitsSuccess implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_GET_SPLITS_SUCCESS;

  constructor(public payload: DashboardSplits) { }
}

export class DashboardSplitsSetActivePeer implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_SET_ACTIVE_PEER;

  constructor(public payload: DashboardSplitsPeer) { }
}

export class DashboardSplitsSplitNodes implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_SPLIT_NODES;
}

export class DashboardSplitsMergeNodesSuccess implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_MERGE_NODES_SUCCESS;
}

export class DashboardSplitsMergeNodes implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_MERGE_NODES;
}

export class DashboardSplitsSplitNodesSuccess implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_SPLIT_NODES_SUCCESS;
}

export class DashboardSplitsSortPeers implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_SORT_PEERS;

  constructor(public payload: TableSort<DashboardSplitsPeer>) { }
}

export class DashboardSplitsToggleSidePanel implements DashboardSplitsAction {
  readonly type = DASHBOARD_SPLITS_TOGGLE_SIDE_PANEL;
}

export type DashboardSplitsActions =
  | DashboardSplitsClose
  | DashboardSplitsGetSplits
  | DashboardSplitsGetSplitsSuccess
  | DashboardSplitsSetActivePeer
  | DashboardSplitsSplitNodes
  | DashboardSplitsSplitNodesSuccess
  | DashboardSplitsMergeNodes
  | DashboardSplitsMergeNodesSuccess
  | DashboardSplitsSortPeers
  | DashboardSplitsToggleSidePanel
  ;
