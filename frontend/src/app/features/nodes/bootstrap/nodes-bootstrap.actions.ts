import { FeatureAction, TableSort } from '@openmina/shared';
import { NodesBootstrapNode } from '@shared/types/nodes/bootstrap/nodes-bootstrap-node.type';

enum NodesBootstrapActionTypes {
  NODES_BOOTSTRAP_INIT = 'NODES_BOOTSTRAP_INIT',
  NODES_BOOTSTRAP_GET_NODES = 'NODES_BOOTSTRAP_GET_NODES',
  NODES_BOOTSTRAP_GET_NODES_SUCCESS = 'NODES_BOOTSTRAP_GET_NODES_SUCCESS',
  NODES_BOOTSTRAP_SORT_NODES = 'NODES_BOOTSTRAP_SORT_NODES',
  NODES_BOOTSTRAP_SET_ACTIVE_BLOCK = 'NODES_BOOTSTRAP_SET_ACTIVE_BLOCK',
  NODES_BOOTSTRAP_TOGGLE_SIDE_PANEL = 'NODES_BOOTSTRAP_TOGGLE_SIDE_PANEL',
  NODES_BOOTSTRAP_CLOSE = 'NODES_BOOTSTRAP_CLOSE',
}

export const NODES_BOOTSTRAP_INIT = NodesBootstrapActionTypes.NODES_BOOTSTRAP_INIT;
export const NODES_BOOTSTRAP_GET_NODES = NodesBootstrapActionTypes.NODES_BOOTSTRAP_GET_NODES;
export const NODES_BOOTSTRAP_GET_NODES_SUCCESS = NodesBootstrapActionTypes.NODES_BOOTSTRAP_GET_NODES_SUCCESS;
export const NODES_BOOTSTRAP_SORT_NODES = NodesBootstrapActionTypes.NODES_BOOTSTRAP_SORT_NODES;
export const NODES_BOOTSTRAP_SET_ACTIVE_BLOCK = NodesBootstrapActionTypes.NODES_BOOTSTRAP_SET_ACTIVE_BLOCK;
export const NODES_BOOTSTRAP_TOGGLE_SIDE_PANEL = NodesBootstrapActionTypes.NODES_BOOTSTRAP_TOGGLE_SIDE_PANEL;
export const NODES_BOOTSTRAP_CLOSE = NodesBootstrapActionTypes.NODES_BOOTSTRAP_CLOSE;

export interface NodesBootstrapAction extends FeatureAction<NodesBootstrapActionTypes> {
  readonly type: NodesBootstrapActionTypes;
}

export class NodesBootstrapInit implements NodesBootstrapAction {
  readonly type = NODES_BOOTSTRAP_INIT;
}

export class NodesBootstrapGetNodes implements NodesBootstrapAction {
  readonly type = NODES_BOOTSTRAP_GET_NODES;

  constructor(public payload?: { force?: boolean }) { }
}

export class NodesBootstrapGetNodesSuccess implements NodesBootstrapAction {
  readonly type = NODES_BOOTSTRAP_GET_NODES_SUCCESS;

  constructor(public payload: NodesBootstrapNode[]) { }
}

export class NodesBootstrapSortNodes implements NodesBootstrapAction {
  readonly type = NODES_BOOTSTRAP_SORT_NODES;

  constructor(public payload: TableSort<NodesBootstrapNode>) { }
}

export class NodesBootstrapSetActiveBlock implements NodesBootstrapAction {
  readonly type = NODES_BOOTSTRAP_SET_ACTIVE_BLOCK;

  constructor(public payload: NodesBootstrapNode) { }
}

export class NodesBootstrapToggleSidePanel implements NodesBootstrapAction {
  readonly type = NODES_BOOTSTRAP_TOGGLE_SIDE_PANEL;
}

export class NodesBootstrapClose implements NodesBootstrapAction {
  readonly type = NODES_BOOTSTRAP_CLOSE;
}

export type NodesBootstrapActions =
  | NodesBootstrapInit
  | NodesBootstrapGetNodes
  | NodesBootstrapGetNodesSuccess
  | NodesBootstrapSortNodes
  | NodesBootstrapSetActiveBlock
  | NodesBootstrapToggleSidePanel
  | NodesBootstrapClose
  ;
