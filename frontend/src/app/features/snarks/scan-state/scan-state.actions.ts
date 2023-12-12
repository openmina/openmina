import { FeatureAction } from '@openmina/shared';
import { ScanStateBlock } from '@shared/types/snarks/scan-state/scan-state-block.type';
import { ScanStateLeaf } from '@shared/types/snarks/scan-state/scan-state-leaf.type';

enum ScanStateActionTypes {
  SCAN_STATE_INIT = 'SCAN_STATE_INIT',
  SCAN_STATE_INIT_SUCCESS = 'SCAN_STATE_INIT_SUCCESS',
  SCAN_STATE_CLOSE = 'SCAN_STATE_CLOSE',
  SCAN_STATE_GET_BLOCK = 'SCAN_STATE_GET_BLOCK',
  SCAN_STATE_GET_BLOCK_SUCCESS = 'SCAN_STATE_GET_BLOCK_SUCCESS',
  SCAN_STATE_SET_ACTIVE_JOB_ID = 'SCAN_STATE_SET_ACTIVE_JOB_ID',
  SCAN_STATE_SET_ACTIVE_LEAF = 'SCAN_STATE_SET_ACTIVE_LEAF',
  SCAN_STATE_TOGGLE_SIDE_PANEL = 'SCAN_STATE_TOGGLE_SIDE_PANEL',
  SCAN_STATE_SIDEBAR_RESIZED = 'SCAN_STATE_SIDEBAR_RESIZED',
  SCAN_STATE_START = 'SCAN_STATE_START',
  SCAN_STATE_PAUSE = 'SCAN_STATE_PAUSE',
  SCAN_STATE_TOGGLE_TREE_VIEW = 'SCAN_STATE_TOGGLE_TREE_VIEW',
  SCAN_STATE_HIGHLIGHT_SNARK_POOL = 'SCAN_STATE_HIGHLIGHT_SNARK_POOL',
}

export const SCAN_STATE_INIT = ScanStateActionTypes.SCAN_STATE_INIT;
export const SCAN_STATE_INIT_SUCCESS = ScanStateActionTypes.SCAN_STATE_INIT_SUCCESS;
export const SCAN_STATE_CLOSE = ScanStateActionTypes.SCAN_STATE_CLOSE;
export const SCAN_STATE_GET_BLOCK = ScanStateActionTypes.SCAN_STATE_GET_BLOCK;
export const SCAN_STATE_GET_BLOCK_SUCCESS = ScanStateActionTypes.SCAN_STATE_GET_BLOCK_SUCCESS;
export const SCAN_STATE_SET_ACTIVE_JOB_ID = ScanStateActionTypes.SCAN_STATE_SET_ACTIVE_JOB_ID;
export const SCAN_STATE_SET_ACTIVE_LEAF = ScanStateActionTypes.SCAN_STATE_SET_ACTIVE_LEAF;
export const SCAN_STATE_TOGGLE_SIDE_PANEL = ScanStateActionTypes.SCAN_STATE_TOGGLE_SIDE_PANEL;
export const SCAN_STATE_SIDEBAR_RESIZED = ScanStateActionTypes.SCAN_STATE_SIDEBAR_RESIZED;
export const SCAN_STATE_START = ScanStateActionTypes.SCAN_STATE_START;
export const SCAN_STATE_PAUSE = ScanStateActionTypes.SCAN_STATE_PAUSE;
export const SCAN_STATE_TOGGLE_TREE_VIEW = ScanStateActionTypes.SCAN_STATE_TOGGLE_TREE_VIEW;
export const SCAN_STATE_HIGHLIGHT_SNARK_POOL = ScanStateActionTypes.SCAN_STATE_HIGHLIGHT_SNARK_POOL;

export interface ScanStateAction extends FeatureAction<ScanStateActionTypes> {
  readonly type: ScanStateActionTypes;
}

export class ScanStateInit implements ScanStateAction {
  readonly type = SCAN_STATE_INIT;
}

export class ScanStateInitSuccess implements ScanStateAction {
  readonly type = SCAN_STATE_INIT_SUCCESS;
}

export class ScanStateClose implements ScanStateAction {
  readonly type = SCAN_STATE_CLOSE;
}

export class ScanStateGetBlock implements ScanStateAction {
  readonly type = SCAN_STATE_GET_BLOCK;

  constructor(public payload: { heightOrHash?: number | string }) {}
}

export class ScanStateGetBlockSuccess implements ScanStateAction {
  readonly type = SCAN_STATE_GET_BLOCK_SUCCESS;

  constructor(public payload: ScanStateBlock) {}
}

export class ScanStateSetActiveJobId implements ScanStateAction {
  readonly type = SCAN_STATE_SET_ACTIVE_JOB_ID;

  constructor(public payload: string) {}
}

export class ScanStateSetActiveLeaf implements ScanStateAction {
  readonly type = SCAN_STATE_SET_ACTIVE_LEAF;

  constructor(public payload: ScanStateLeaf) {}
}

export class ScanStateToggleSidePanel implements ScanStateAction {
  readonly type = SCAN_STATE_TOGGLE_SIDE_PANEL;
}

export class ScanStateSidebarResized implements ScanStateAction {
  readonly type = SCAN_STATE_SIDEBAR_RESIZED;
}

export class ScanStateStart implements ScanStateAction {
  readonly type = SCAN_STATE_START;
}

export class ScanStatePause implements ScanStateAction {
  readonly type = SCAN_STATE_PAUSE;
}

export class ScanStateToggleTreeView implements ScanStateAction {
  readonly type = SCAN_STATE_TOGGLE_TREE_VIEW;
}

export class ScanStateHighlightSnarkPool implements ScanStateAction {
  readonly type = SCAN_STATE_HIGHLIGHT_SNARK_POOL;
}

export type ScanStateActions =
  | ScanStateInit
  | ScanStateInitSuccess
  | ScanStateClose
  | ScanStateGetBlock
  | ScanStateGetBlockSuccess
  | ScanStateSetActiveJobId
  | ScanStateSetActiveLeaf
  | ScanStateToggleSidePanel
  | ScanStateSidebarResized
  | ScanStateStart
  | ScanStatePause
  | ScanStateToggleTreeView
  | ScanStateHighlightSnarkPool
  ;
