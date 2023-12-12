import { DashboardPeer } from '@shared/types/dashboard/dashboard.peer';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { DashboardPeersStats } from '@shared/types/dashboard/dashboard-peers-stats.type';
import { TableSort } from '@openmina/shared';
import { DashboardPeersSort } from '@dashboard/dashboard.actions';

export interface DashboardState {
  peers: DashboardPeer[];
  peersStats: DashboardPeersStats;
  peersSort: TableSort<DashboardPeer>;
  nodeBootstrappingPercentage: number;
  appliedBlocks: number;
  maxBlockHeightSeen: number;
  berkeleyBlockHeight: number;
  receivedBlocks: number;
  receivedTxs: number;
  receivedSnarks: number;
}

const select = <T>(selector: (state: DashboardState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectDashboardState,
  selector,
);

export const selectDashboardState = createFeatureSelector<DashboardState>('dashboard');
export const selectDashboardPeers = select((state: DashboardState): DashboardPeer[] => state.peers);
export const selectDashboardPeersStats = select((state: DashboardState): DashboardPeersStats => state.peersStats);
export const selectDashboardPeersSort = select((state: DashboardState): TableSort<DashboardPeer> => state.peersSort);
