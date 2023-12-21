import { NetworkConnection } from '@shared/types/network/connections/network-connection.type';
import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';
import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectNetworkConnectionsState } from '@network/network.state';

export interface NetworkConnectionsState {
  connections: NetworkConnection[];
  activeConnection: NetworkConnection;
  stream: boolean;
  limit: number;
  direction: NetworkMessagesDirection;
}

const select = <T>(selector: (state: NetworkConnectionsState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkConnectionsState,
  selector,
);

export const selectNetworkConnections = select((state: NetworkConnectionsState): NetworkConnection[] => state.connections);
export const selectNetworkConnectionsActiveConnection = select((state: NetworkConnectionsState): NetworkConnection => state.activeConnection);
