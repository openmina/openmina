import { MinaState } from '@app/app.setup';
import { NetworkMessage } from '@shared/types/network/messages/network-message.type';
import { NetworkMessageConnection } from '@shared/types/network/messages/network-messages-connection.type';
import { TimestampInterval, VirtualScrollActivePage } from '@openmina/shared';
import { NetworkMessagesFilter } from '@shared/types/network/messages/network-messages-filter.type';
import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';
import { createSelector, MemoizedSelector } from '@ngrx/store';
import { selectNetworkMessagesState } from '@network/network.state';

export interface NetworkMessagesState {
  messages: NetworkMessage[];
  activeRow: NetworkMessage;
  activeRowFullMessage: any;
  activeRowHex: string;
  connection: NetworkMessageConnection;
  activeFilters: NetworkMessagesFilter[];
  timestamp: TimestampInterval;
  activeTab: number;
  stream: boolean;
  limit: number;
  direction: NetworkMessagesDirection;
  activePage: VirtualScrollActivePage<NetworkMessage>;
  pages: number[];
}

const select = <T>(selector: (state: NetworkMessagesState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkMessagesState,
  selector,
);

export const selectNetworkStream = select((network: NetworkMessagesState): boolean => network.stream);
export const selectNetworkMessages = select((network: NetworkMessagesState): NetworkMessage[] => network.messages);
export const selectNetworkActiveRow = select((network: NetworkMessagesState): NetworkMessage => network.activeRow);
export const selectNetworkMessageHex = select((network: NetworkMessagesState): string => network.activeRowHex);
export const selectNetworkFullMessage = select((network: NetworkMessagesState): any => network.activeRowFullMessage);
export const selectNetworkConnection = select((network: NetworkMessagesState): NetworkMessageConnection => network.connection);
export const selectNetworkActiveFilters = select((network: NetworkMessagesState): NetworkMessagesFilter[] => network.activeFilters);
export const selectNetworkTimestampInterval = select((network: NetworkMessagesState): TimestampInterval => network.timestamp);
