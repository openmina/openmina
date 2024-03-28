import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { NetworkMessagesState } from '@network/messages/network-messages.state';
import { NetworkConnectionsState } from '@network/connections/network-connections.state';
import { NetworkBlocksState } from '@network/blocks/network-blocks.state';
import { DashboardSplitsState } from '@network/splits/dashboard-splits.state';
import { NetworkNodeDhtState } from '@network/node-dht/network-node-dht.state';
import { NetworkBootstrapStatsState } from '@network/bootstrap-stats/network-bootstrap-stats.state';

export interface NetworkState {
  messages: NetworkMessagesState;
  connections: NetworkConnectionsState;
  blocks: NetworkBlocksState;
  splits: DashboardSplitsState;
  nodeDht: NetworkNodeDhtState;
  bootstrapStats: NetworkBootstrapStatsState;
}

const select = <T>(selector: (state: NetworkState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkState,
  selector,
);

export const selectNetworkState = createFeatureSelector<NetworkState>('network');
export const selectNetworkMessagesState = select((state: NetworkState): NetworkMessagesState => state.messages);
export const selectNetworkConnectionsState = select((state: NetworkState): NetworkConnectionsState => state.connections);
export const selectNetworkBlocksState = select((state: NetworkState): NetworkBlocksState => state.blocks);
export const selectDashboardSplitsState = select((state: NetworkState): DashboardSplitsState => state.splits);
export const selectNetworkNodeDhtState = select((state: NetworkState): NetworkNodeDhtState => state.nodeDht);
export const selectNetworkBootstrapStatsState = select((state: NetworkState): NetworkBootstrapStatsState => state.bootstrapStats);
