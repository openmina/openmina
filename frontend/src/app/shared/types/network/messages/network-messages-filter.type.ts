import { NetworkMessagesFilterTypes } from '@shared/types/network/messages/network-messages-filter-types.enum';

export interface NetworkMessagesFilter {
  type: NetworkMessagesFilterTypes;
  display: string;
  value: string | number;
  tooltip?: string;
}
