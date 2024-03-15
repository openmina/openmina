import { DashboardPeer } from '@shared/types/dashboard/dashboard.peer';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { DashboardPeersStats } from '@shared/types/dashboard/dashboard-peers-stats.type';
import { TableSort } from '@openmina/shared';
import { DashboardPeersSort } from '@dashboard/dashboard.actions';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { DashboardRpcStats } from '@shared/types/dashboard/dashboard-rpc-stats.type';

export interface DashboardState {
  peers: DashboardPeer[];
  peersStats: DashboardPeersStats;
  peersSort: TableSort<DashboardPeer>;
  nodes: NodesOverviewNode[];
  rpcStats: DashboardRpcStats;
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
export const selectDashboardNodes = select((state: DashboardState): NodesOverviewNode[] => state.nodes);
export const selectDashboardNodesAndPeers = select((state: DashboardState): [NodesOverviewNode[], DashboardPeer[]] => [state.nodes, state.peers]);
export const selectDashboardRpcStats = select((state: DashboardState): DashboardRpcStats => state.rpcStats);
export const selectDashboardNodesAndRpcStats = select((state: DashboardState): [NodesOverviewNode[], DashboardRpcStats] => [state.nodes, state.rpcStats]);
