import { createType } from '@shared/constants/store-functions';
import { createAction, props } from '@ngrx/store';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { TableSort } from '@openmina/shared';

export const LEADERBOARD_KEY = 'leaderboard';
export const LEADERBOARD_PREFIX = 'Leaderboard';

const type = <T extends string>(type: T) => createType(LEADERBOARD_PREFIX, null, type);

const init = createAction(type('Init'));
const close = createAction(type('Close'));
const getHeartbeats = createAction(type('Get Heartbeats'));
const getHeartbeatsSuccess = createAction(type('Get Heartbeats Success'), props<{
  heartbeatSummaries: HeartbeatSummary[],
}>());
const changeFilters = createAction(type('Change Filters'), props<{ filters: any }>());
const sort = createAction(type('Sort'), props<{ sort: TableSort<HeartbeatSummary> }>());

export const LeaderboardActions = {
  init,
  close,
  getHeartbeats,
  getHeartbeatsSuccess,
  changeFilters,
  sort,
};
