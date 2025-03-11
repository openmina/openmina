import { createReducer, on } from '@ngrx/store';
import { LeaderboardState } from '@leaderboard/leaderboard.state';
import { LeaderboardActions } from '@leaderboard/leaderboard.actions';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { sort, SortDirection, TableSort } from '@openmina/shared';


const initialState: LeaderboardState = {
  filteredHeartbeatSummaries: [],
  heartbeatSummaries: [],
  filters: {
    search: '',
  },
  sortBy: {
    sortDirection: SortDirection.DSC,
    sortBy: 'uptimePercentage',
  },
  isLoading: true,
};

export const leaderboardReducer = createReducer(
  initialState,
  on(LeaderboardActions.getHeartbeatsSuccess, (state, { heartbeatSummaries }) => ({
    ...state,
    isLoading: false,
    heartbeatSummaries,
    filteredHeartbeatSummaries: sortHeartbeats(filterHeartbeats(heartbeatSummaries, state.filters), state.sortBy),
  })),
  on(LeaderboardActions.changeFilters, (state, { filters }) => ({
    ...state,
    filters,
    filteredHeartbeatSummaries: sortHeartbeats(filterHeartbeats(state.heartbeatSummaries, filters), state.sortBy),
  })),
  on(LeaderboardActions.sort, (state, { sort }) => ({
    ...state,
    sortBy: sort,
    filteredHeartbeatSummaries: sortHeartbeats(state.filteredHeartbeatSummaries, sort),
  })),
);


function sortHeartbeats(node: HeartbeatSummary[], tableSort: TableSort<HeartbeatSummary>): HeartbeatSummary[] {
  const data = sort<HeartbeatSummary>(node, tableSort, []);
  const whales = data.filter(d => d.isWhale).sort((a, b) => b.uptimePercentage - a.uptimePercentage);
  const nonWhales = data.filter(d => !d.isWhale);
  return [...nonWhales, ...whales];
}

function filterHeartbeats(summaries: HeartbeatSummary[], filters: any): HeartbeatSummary[] {
  return summaries.filter(summary => {
    if (filters.search?.length) {
      const searchTerm = filters.search.toLowerCase();
      const searchMatch = summary.publicKey.toLowerCase().includes(searchTerm);
      if (!searchMatch) return false;
    }

    return true;
  });
}
