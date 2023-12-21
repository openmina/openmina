import { ActionReducer, combineReducers } from '@ngrx/store';
import { NetworkState } from '@network/network.state';

import * as fromMessages from '@network/messages/network-messages.reducer';
import * as fromConnections from '@network/connections/network-connections.reducer';
import * as fromBlocks from '@network/blocks/network-blocks.reducer';

import { NetworkMessagesAction, NetworkMessagesActions } from '@network/messages/network-messages.actions';
import { NetworkConnectionsAction, NetworkConnectionsActions } from '@network/connections/network-connections.actions';
import { NetworkBlocksAction, NetworkBlocksActions } from '@network/blocks/network-blocks.actions';

export type NetworkActions = NetworkMessagesActions & NetworkConnectionsActions & NetworkBlocksActions;
export type NetworkAction = NetworkMessagesAction & NetworkConnectionsAction & NetworkBlocksAction;

export const networkReducer: ActionReducer<NetworkState, NetworkActions> = combineReducers<NetworkState, NetworkActions>({
  messages: fromMessages.reducer,
  connections: fromConnections.reducer,
  blocks: fromBlocks.reducer,
});
