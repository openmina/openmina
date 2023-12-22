import { FeatureAction, TableSort } from '@openmina/shared';
import { WorkPool } from '@shared/types/snarks/work-pool/work-pool.type';
import { WorkPoolSpecs } from '@shared/types/snarks/work-pool/work-pool-specs.type';
import { WorkPoolDetail } from '@shared/types/snarks/work-pool/work-pool-detail.type';

enum SnarksWorkPoolTypes {
  SNARKS_WORK_POOL_INIT = 'SNARKS_WORK_POOL_INIT',
  SNARKS_WORK_POOL_GET_WORK_POOL = 'SNARKS_WORK_POOL_GET_WORK_POOL',
  SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS = 'SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS',
  SNARKS_WORK_POOL_SORT_WORK_POOL = 'SNARKS_WORK_POOL_SORT_WORK_POOL',
  SNARKS_WORK_POOL_SET_ACTIVE_WORK_POOL = 'SNARKS_WORK_POOL_SET_ACTIVE_WORK_POOL',
  SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL = 'SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL',
  SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS = 'SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS',
  SNARKS_WORK_POOL_TOGGLE_SIDE_PANEL = 'SNARKS_WORK_POOL_TOGGLE_SIDE_PANEL',
  SNARKS_WORK_POOL_TOGGLE_FILTER = 'SNARKS_WORK_POOL_TOGGLE_FILTER',
  SNARKS_WORK_POOL_CLOSE = 'SNARKS_WORK_POOL_CLOSE',
}

export const SNARKS_WORK_POOL_INIT = SnarksWorkPoolTypes.SNARKS_WORK_POOL_INIT;
export const SNARKS_WORK_POOL_GET_WORK_POOL = SnarksWorkPoolTypes.SNARKS_WORK_POOL_GET_WORK_POOL;
export const SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS = SnarksWorkPoolTypes.SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS;
export const SNARKS_WORK_POOL_SORT_WORK_POOL = SnarksWorkPoolTypes.SNARKS_WORK_POOL_SORT_WORK_POOL;
export const SNARKS_WORK_POOL_SET_ACTIVE_WORK_POOL = SnarksWorkPoolTypes.SNARKS_WORK_POOL_SET_ACTIVE_WORK_POOL;
export const SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL = SnarksWorkPoolTypes.SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL;
export const SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS = SnarksWorkPoolTypes.SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS;
export const SNARKS_WORK_POOL_TOGGLE_SIDE_PANEL = SnarksWorkPoolTypes.SNARKS_WORK_POOL_TOGGLE_SIDE_PANEL;
export const SNARKS_WORK_POOL_TOGGLE_FILTER = SnarksWorkPoolTypes.SNARKS_WORK_POOL_TOGGLE_FILTER;
export const SNARKS_WORK_POOL_CLOSE = SnarksWorkPoolTypes.SNARKS_WORK_POOL_CLOSE;

export interface SnarksWorkPoolAction extends FeatureAction<SnarksWorkPoolTypes> {
  readonly type: SnarksWorkPoolTypes;
}

export class SnarksWorkPoolInit implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_INIT;
}

export class SnarksWorkPoolGetWorkPool implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_GET_WORK_POOL;

  constructor(public payload?: { force?: boolean }) { }
}

export class SnarksWorkPoolGetWorkPoolSuccess implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS;

  constructor(public payload: WorkPool[]) { }
}

export class SnarksWorkPoolSortWorkPool implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_SORT_WORK_POOL;

  constructor(public payload: TableSort<WorkPool>) { }
}

export class SnarksWorkPoolSetActiveWorkPool implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_SET_ACTIVE_WORK_POOL;

  constructor(public payload: { id: string }) { }
}

export class SnarksWorkPoolGetWorkPoolDetail implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL;

  constructor(public payload: { id: string }) { }
}

export class SnarksWorkPoolGetWorkPoolDetailSuccess implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS;

  constructor(public payload: [WorkPoolSpecs, WorkPoolDetail]) { }
}

export class SnarksWorkPoolToggleSidePanel implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_TOGGLE_SIDE_PANEL;
}

export class SnarksWorkPoolToggleFilter implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_TOGGLE_FILTER;

  constructor(public payload: string) { }
}

export class SnarksWorkPoolClose implements SnarksWorkPoolAction {
  readonly type = SNARKS_WORK_POOL_CLOSE;
}

export type SnarksWorkPoolActions =
  | SnarksWorkPoolInit
  | SnarksWorkPoolGetWorkPool
  | SnarksWorkPoolGetWorkPoolSuccess
  | SnarksWorkPoolSortWorkPool
  | SnarksWorkPoolSetActiveWorkPool
  | SnarksWorkPoolGetWorkPoolDetail
  | SnarksWorkPoolGetWorkPoolDetailSuccess
  | SnarksWorkPoolToggleSidePanel
  | SnarksWorkPoolToggleFilter
  | SnarksWorkPoolClose
  ;
