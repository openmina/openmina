import { FeatureAction, TableSort } from '@openmina/shared';
import { NodesLiveNode } from '@shared/types/nodes/live/nodes-live-node.type';
import { NodesLiveBlockEvent } from '@shared/types/nodes/live/nodes-live-block-event.type';

enum NodesLiveActionTypes {
  NODES_LIVE_INIT = 'NODES_LIVE_INIT',
  NODES_LIVE_GET_NODES = 'NODES_LIVE_GET_NODES',
  NODES_LIVE_GET_NODES_SUCCESS = 'NODES_LIVE_GET_NODES_SUCCESS',
  NODES_LIVE_SORT_EVENTS = 'NODES_LIVE_SORT_EVENTS',
  NODES_LIVE_SET_ACTIVE_NODE = 'NODES_LIVE_SET_ACTIVE_NODE',
  NODES_LIVE_TOGGLE_SIDE_PANEL = 'NODES_LIVE_TOGGLE_SIDE_PANEL',
  NODES_LIVE_TOGGLE_FILTER = 'NODES_LIVE_TOGGLE_FILTER',
  NODES_LIVE_CLOSE = 'NODES_LIVE_CLOSE',
}

export const NODES_LIVE_INIT = NodesLiveActionTypes.NODES_LIVE_INIT;
export const NODES_LIVE_GET_NODES = NodesLiveActionTypes.NODES_LIVE_GET_NODES;
export const NODES_LIVE_GET_NODES_SUCCESS = NodesLiveActionTypes.NODES_LIVE_GET_NODES_SUCCESS;
export const NODES_LIVE_SORT_EVENTS = NodesLiveActionTypes.NODES_LIVE_SORT_EVENTS;
export const NODES_LIVE_SET_ACTIVE_NODE = NodesLiveActionTypes.NODES_LIVE_SET_ACTIVE_NODE;
export const NODES_LIVE_TOGGLE_SIDE_PANEL = NodesLiveActionTypes.NODES_LIVE_TOGGLE_SIDE_PANEL;
export const NODES_LIVE_TOGGLE_FILTER = NodesLiveActionTypes.NODES_LIVE_TOGGLE_FILTER;
export const NODES_LIVE_CLOSE = NodesLiveActionTypes.NODES_LIVE_CLOSE;

export interface NodesLiveAction extends FeatureAction<NodesLiveActionTypes> {
  readonly type: NodesLiveActionTypes;
}

export class NodesLiveInit implements NodesLiveAction {
  readonly type = NODES_LIVE_INIT;
}

export class NodesLiveGetNodes implements NodesLiveAction {
  readonly type = NODES_LIVE_GET_NODES;

  constructor(public payload?: { force?: boolean }) { }
}

export class NodesLiveGetNodesSuccess implements NodesLiveAction {
  readonly type = NODES_LIVE_GET_NODES_SUCCESS;

  constructor(public payload: NodesLiveNode[]) { }
}

export class NodesLiveSortEvents implements NodesLiveAction {
  readonly type = NODES_LIVE_SORT_EVENTS;

  constructor(public payload: TableSort<NodesLiveBlockEvent>) { }
}

export class NodesLiveSetActiveNode implements NodesLiveAction {
  readonly type = NODES_LIVE_SET_ACTIVE_NODE;

  constructor(public payload: { hash: string }) { }
}

export class NodesLiveToggleSidePanel implements NodesLiveAction {
  readonly type = NODES_LIVE_TOGGLE_SIDE_PANEL;
}

export class NodesLiveToggleFilter implements NodesLiveAction {
  readonly type = NODES_LIVE_TOGGLE_FILTER;

  constructor(public payload: string) { }
}

export class NodesLiveClose implements NodesLiveAction {
  readonly type = NODES_LIVE_CLOSE;
}

export type NodesLiveActions =
  | NodesLiveInit
  | NodesLiveGetNodes
  | NodesLiveGetNodesSuccess
  | NodesLiveSortEvents
  | NodesLiveSetActiveNode
  | NodesLiveToggleSidePanel
  | NodesLiveToggleFilter
  | NodesLiveClose
  ;
