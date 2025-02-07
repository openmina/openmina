import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { LEADERBOARD_KEY } from '@leaderboard/leaderboard.actions';
import { TableSort } from '@openmina/shared';

export interface LeaderboardState {
  filteredHeartbeatSummaries: HeartbeatSummary[];
  heartbeatSummaries: HeartbeatSummary[];
  filters: { search: string };
  sortBy: TableSort<HeartbeatSummary>;
  isLoading: boolean;
}

const select = <T>(selector: (state: LeaderboardState) => T): MemoizedSelector<MinaState, T> => createSelector(
  createFeatureSelector<LeaderboardState>(LEADERBOARD_KEY),
  selector,
);
const filteredHeartbeatSummaries = select(state => state.filteredHeartbeatSummaries);
const filters = select(state => state.filters);
const sortBy = select(state => state.sortBy);
const isLoading = select(state => state.isLoading);

export const LeaderboardSelectors = {
  filteredHeartbeatSummaries,
  filters,
  sortBy,
  isLoading,
};
