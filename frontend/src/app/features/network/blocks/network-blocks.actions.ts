import { FeatureAction, TableSort } from '@openmina/shared';
import { NetworkBlock } from '@shared/types/network/blocks/network-block.type';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

enum NetworkBlocksActionTypes {
  NETWORK_BLOCKS_INIT = 'NETWORK_BLOCKS_INIT',
  NETWORK_BLOCKS_CLOSE = 'NETWORK_BLOCKS_CLOSE',
  NETWORK_BLOCKS_GET_BLOCKS = 'NETWORK_BLOCKS_GET_BLOCKS',
  NETWORK_BLOCKS_GET_BLOCKS_SUCCESS = 'NETWORK_BLOCKS_GET_BLOCKS_SUCCESS',
  NETWORK_BLOCKS_SORT = 'NETWORK_BLOCKS_SORT',
  NETWORK_BLOCKS_TOGGLE_SIDE_PANEL = 'NETWORK_BLOCKS_TOGGLE_SIDE_PANEL',
  NETWORK_BLOCKS_TOGGLE_FILTER = 'NETWORK_BLOCKS_TOGGLE_FILTER',
  NETWORK_BLOCKS_SET_ACTIVE_BLOCK = 'NETWORK_BLOCKS_SET_ACTIVE_BLOCK',
  NETWORK_BLOCKS_GET_EARLIEST_BLOCK = 'NETWORK_BLOCKS_GET_EARLIEST_BLOCK',
  NETWORK_BLOCKS_SET_EARLIEST_BLOCK = 'NETWORK_BLOCKS_SET_EARLIEST_BLOCK',
}

export const NETWORK_BLOCKS_INIT = NetworkBlocksActionTypes.NETWORK_BLOCKS_INIT;
export const NETWORK_BLOCKS_CLOSE = NetworkBlocksActionTypes.NETWORK_BLOCKS_CLOSE;
export const NETWORK_BLOCKS_GET_BLOCKS = NetworkBlocksActionTypes.NETWORK_BLOCKS_GET_BLOCKS;
export const NETWORK_BLOCKS_GET_BLOCKS_SUCCESS = NetworkBlocksActionTypes.NETWORK_BLOCKS_GET_BLOCKS_SUCCESS;
export const NETWORK_BLOCKS_SORT = NetworkBlocksActionTypes.NETWORK_BLOCKS_SORT;
export const NETWORK_BLOCKS_TOGGLE_SIDE_PANEL = NetworkBlocksActionTypes.NETWORK_BLOCKS_TOGGLE_SIDE_PANEL;
export const NETWORK_BLOCKS_TOGGLE_FILTER = NetworkBlocksActionTypes.NETWORK_BLOCKS_TOGGLE_FILTER;
export const NETWORK_BLOCKS_SET_ACTIVE_BLOCK = NetworkBlocksActionTypes.NETWORK_BLOCKS_SET_ACTIVE_BLOCK;
export const NETWORK_BLOCKS_GET_EARLIEST_BLOCK = NetworkBlocksActionTypes.NETWORK_BLOCKS_GET_EARLIEST_BLOCK;
export const NETWORK_BLOCKS_SET_EARLIEST_BLOCK = NetworkBlocksActionTypes.NETWORK_BLOCKS_SET_EARLIEST_BLOCK;

export interface NetworkBlocksAction extends FeatureAction<NetworkBlocksActionTypes> {
  readonly type: NetworkBlocksActionTypes;
}

export class NetworkBlocksInit implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_INIT;
}

export class NetworkBlocksClose implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_CLOSE;
}

export class NetworkBlocksGetBlocks implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_GET_BLOCKS;

  constructor(public payload?: { height: number }) {}
}

export class NetworkBlocksGetBlocksSuccess implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_GET_BLOCKS_SUCCESS;

  constructor(public payload: NetworkBlock[]) {}
}

export class NetworkBlocksSort implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_SORT;

  constructor(public payload: TableSort<NetworkBlock>) { }
}

export class NetworkBlocksToggleSidePanel implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_TOGGLE_SIDE_PANEL;
}

export class NetworkBlocksToggleFilter implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_TOGGLE_FILTER;

  constructor(public payload: string) { }
}

export class NetworkBlocksSetActiveBlock implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_SET_ACTIVE_BLOCK;

  constructor(public payload: { height: number, fetchNew?: boolean }) { }
}

export class NetworkBlocksGetEarliestBlock implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_GET_EARLIEST_BLOCK;

  constructor(public payload: MinaNode) { }
}

export class NetworkBlocksSetEarliestBlock implements NetworkBlocksAction {
  readonly type = NETWORK_BLOCKS_SET_EARLIEST_BLOCK;

  constructor(public payload: { height: number }) { }
}


export type NetworkBlocksActions =
  | NetworkBlocksInit
  | NetworkBlocksClose
  | NetworkBlocksGetBlocks
  | NetworkBlocksGetBlocksSuccess
  | NetworkBlocksSort
  | NetworkBlocksToggleSidePanel
  | NetworkBlocksToggleFilter
  | NetworkBlocksSetActiveBlock
  | NetworkBlocksGetEarliestBlock
  | NetworkBlocksSetEarliestBlock
  ;
