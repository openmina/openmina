import { NetworkMessagesFilter } from '@shared/types/network/messages/network-messages-filter.type';

export interface NetworkMessagesFilterCategory {
  name: string;
  filters: NetworkMessagesFilter[];
  tooltip: string;
}
