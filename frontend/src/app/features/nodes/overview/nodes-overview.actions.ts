import { FeatureAction, TableSort } from '@openmina/shared';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

enum NodesOverviewActionTypes {
  NODES_OVERVIEW_INIT = 'NODES_OVERVIEW_INIT',
  NODES_OVERVIEW_GET_NODES = 'NODES_OVERVIEW_GET_NODES',
  NODES_OVERVIEW_GET_NODES_SUCCESS = 'NODES_OVERVIEW_GET_NODES_SUCCESS',
  NODES_OVERVIEW_GET_NODE_STATUS = 'NODES_OVERVIEW_GET_NODE_STATUS',
  NODES_OVERVIEW_GET_NODE_STATUS_SUCCESS = 'NODES_OVERVIEW_GET_NODE_STATUS_SUCCESS',
  NODES_OVERVIEW_SORT_NODES = 'NODES_OVERVIEW_SORT_NODES',
  NODES_OVERVIEW_SET_ACTIVE_NODE = 'NODES_OVERVIEW_SET_ACTIVE_NODE',
  NODES_OVERVIEW_CLOSE = 'NODES_OVERVIEW_CLOSE',
}

export const NODES_OVERVIEW_INIT = NodesOverviewActionTypes.NODES_OVERVIEW_INIT;
export const NODES_OVERVIEW_GET_NODES = NodesOverviewActionTypes.NODES_OVERVIEW_GET_NODES;
export const NODES_OVERVIEW_GET_NODES_SUCCESS = NodesOverviewActionTypes.NODES_OVERVIEW_GET_NODES_SUCCESS;
export const NODES_OVERVIEW_GET_NODE_STATUS = NodesOverviewActionTypes.NODES_OVERVIEW_GET_NODE_STATUS;
export const NODES_OVERVIEW_GET_NODE_STATUS_SUCCESS = NodesOverviewActionTypes.NODES_OVERVIEW_GET_NODE_STATUS_SUCCESS;
export const NODES_OVERVIEW_SORT_NODES = NodesOverviewActionTypes.NODES_OVERVIEW_SORT_NODES;
export const NODES_OVERVIEW_SET_ACTIVE_NODE = NodesOverviewActionTypes.NODES_OVERVIEW_SET_ACTIVE_NODE;
export const NODES_OVERVIEW_CLOSE = NodesOverviewActionTypes.NODES_OVERVIEW_CLOSE;

export interface NodesOverviewAction extends FeatureAction<NodesOverviewActionTypes> {
  readonly type: NodesOverviewActionTypes;
}

export class NodesOverviewInit implements NodesOverviewAction {
  readonly type = NODES_OVERVIEW_INIT;
}

export class NodesOverviewGetNodes implements NodesOverviewAction {
  readonly type = NODES_OVERVIEW_GET_NODES;
}

export class NodesOverviewGetNodesSuccess implements NodesOverviewAction {
  readonly type = NODES_OVERVIEW_GET_NODES_SUCCESS;

  constructor(public payload: NodesOverviewNode[]) { }
}

export class NodesOverviewGetNodeStatus implements NodesOverviewAction {
  readonly type = NODES_OVERVIEW_GET_NODE_STATUS;

  constructor(public payload: MinaNode) {}
}

export class NodesOverviewGetNodeStatusSuccess implements NodesOverviewAction {
  readonly type = NODES_OVERVIEW_GET_NODE_STATUS_SUCCESS;

  constructor(public payload: NodesOverviewNode) { }
}

export class NodesOverviewSortNodes implements NodesOverviewAction {
  readonly type = NODES_OVERVIEW_SORT_NODES;

  constructor(public payload: TableSort<NodesOverviewNode>) { }
}

export class NodesOverviewSetActiveNode implements NodesOverviewAction {
  readonly type = NODES_OVERVIEW_SET_ACTIVE_NODE;

  constructor(public payload: NodesOverviewNode) { }
}

export class NodesOverviewClose implements NodesOverviewAction {
  readonly type = NODES_OVERVIEW_CLOSE;
}

export type NodesOverviewActions =
  | NodesOverviewInit
  | NodesOverviewGetNodes
  | NodesOverviewGetNodesSuccess
  | NodesOverviewGetNodeStatus
  | NodesOverviewGetNodeStatusSuccess
  | NodesOverviewSortNodes
  | NodesOverviewSetActiveNode
  | NodesOverviewClose
  ;
