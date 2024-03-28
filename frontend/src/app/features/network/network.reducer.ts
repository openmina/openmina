import { ActionReducer, combineReducers } from '@ngrx/store';
import { NetworkState } from '@network/network.state';

import { NetworkMessagesAction, NetworkMessagesActions } from '@network/messages/network-messages.actions';
import { NetworkConnectionsAction, NetworkConnectionsActions } from '@network/connections/network-connections.actions';
import { NetworkBlocksAction, NetworkBlocksActions } from '@network/blocks/network-blocks.actions';
import { NetworkNodeDhtAction, NetworkNodeDhtActions } from '@network/node-dht/network-node-dht.actions';
import {
  NetworkBootstrapStatsAction,
  NetworkBootstrapStatsActions,
} from '@network/bootstrap-stats/network-bootstrap-stats.actions';
import { networkMessagesReducer } from '@network/messages/network-messages.reducer';
import { networkConnectionsReducer } from '@network/connections/network-connections.reducer';
import { networkBlocksReducer } from '@network/blocks/network-blocks.reducer';
import { topologyReducer } from '@network/splits/dashboard-splits.reducer';
import { networkDhtReducer } from '@network/node-dht/network-node-dht.reducer';
import { networkBootstrapStatsReducer } from '@network/bootstrap-stats/network-bootstrap-stats.reducer';

export type NetworkActions =
  & NetworkMessagesActions
  & NetworkConnectionsActions
  & NetworkBlocksActions
  & NetworkNodeDhtActions
  & NetworkBootstrapStatsActions;

export type NetworkAction =
  & NetworkMessagesAction
  & NetworkConnectionsAction
  & NetworkBlocksAction
  & NetworkNodeDhtAction
  & NetworkBootstrapStatsAction;

export const networkReducer: ActionReducer<NetworkState, NetworkActions> = combineReducers<NetworkState, NetworkActions>({
  messages: networkMessagesReducer,
  connections: networkConnectionsReducer,
  blocks: networkBlocksReducer,
  splits: topologyReducer,
  nodeDht: networkDhtReducer,
  bootstrapStats: networkBootstrapStatsReducer,
});
