import { FeatureAction, TimestampInterval } from '@openmina/shared';
import { NetworkMessage } from '@shared/types/network/messages/network-message.type';
import { NetworkMessageConnection } from '@shared/types/network/messages/network-messages-connection.type';
import { NetworkMessagesFilter } from '@shared/types/network/messages/network-messages-filter.type';
import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';

enum NetworkMessagesActionTypes {
  NETWORK_INIT = 'NETWORK_INIT',
  NETWORK_CLOSE = 'NETWORK_CLOSE',
  NETWORK_GET_MESSAGES = 'NETWORK_GET_MESSAGES',
  NETWORK_GET_MESSAGES_SUCCESS = 'NETWORK_GET_MESSAGES_SUCCESS',
  NETWORK_GET_PAGINATED_MESSAGES = 'NETWORK_GET_PAGINATED_MESSAGES',
  NETWORK_TOGGLE_FILTER = 'NETWORK_TOGGLE_FILTER',
  NETWORK_SET_TIMESTAMP_INTERVAL = 'NETWORK_SET_TIMESTAMP_INTERVAL',
  NETWORK_SET_ACTIVE_ROW = 'NETWORK_SET_ACTIVE_ROW',
  NETWORK_GO_LIVE = 'NETWORK_GO_LIVE',
  NETWORK_PAUSE = 'NETWORK_PAUSE',
  NETWORK_GET_FULL_MESSAGE = 'NETWORK_GET_FULL_MESSAGE',
  NETWORK_GET_FULL_MESSAGE_SUCCESS = 'NETWORK_GET_FULL_MESSAGE_SUCCESS',
  NETWORK_GET_MESSAGE_HEX = 'NETWORK_GET_MESSAGE_HEX',
  NETWORK_GET_MESSAGE_HEX_SUCCESS = 'NETWORK_GET_MESSAGE_HEX_SUCCESS',
  NETWORK_GET_CONNECTION = 'NETWORK_GET_CONNECTION',
  NETWORK_GET_CONNECTION_SUCCESS = 'NETWORK_GET_CONNECTION_SUCCESS',
  NETWORK_CHANGE_TAB = 'NETWORK_CHANGE_TAB',
  NETWORK_GET_SPECIFIC_MESSAGE = 'NETWORK_GET_SPECIFIC_MESSAGE',
}

export const NETWORK_INIT = NetworkMessagesActionTypes.NETWORK_INIT;
export const NETWORK_CLOSE = NetworkMessagesActionTypes.NETWORK_CLOSE;
export const NETWORK_GET_MESSAGES = NetworkMessagesActionTypes.NETWORK_GET_MESSAGES;
export const NETWORK_GET_MESSAGES_SUCCESS = NetworkMessagesActionTypes.NETWORK_GET_MESSAGES_SUCCESS;
export const NETWORK_GET_PAGINATED_MESSAGES = NetworkMessagesActionTypes.NETWORK_GET_PAGINATED_MESSAGES;
export const NETWORK_TOGGLE_FILTER = NetworkMessagesActionTypes.NETWORK_TOGGLE_FILTER;
export const NETWORK_SET_TIMESTAMP_INTERVAL = NetworkMessagesActionTypes.NETWORK_SET_TIMESTAMP_INTERVAL;
export const NETWORK_SET_ACTIVE_ROW = NetworkMessagesActionTypes.NETWORK_SET_ACTIVE_ROW;
export const NETWORK_GO_LIVE = NetworkMessagesActionTypes.NETWORK_GO_LIVE;
export const NETWORK_PAUSE = NetworkMessagesActionTypes.NETWORK_PAUSE;
export const NETWORK_GET_FULL_MESSAGE = NetworkMessagesActionTypes.NETWORK_GET_FULL_MESSAGE;
export const NETWORK_GET_FULL_MESSAGE_SUCCESS = NetworkMessagesActionTypes.NETWORK_GET_FULL_MESSAGE_SUCCESS;
export const NETWORK_GET_MESSAGE_HEX = NetworkMessagesActionTypes.NETWORK_GET_MESSAGE_HEX;
export const NETWORK_GET_MESSAGE_HEX_SUCCESS = NetworkMessagesActionTypes.NETWORK_GET_MESSAGE_HEX_SUCCESS;
export const NETWORK_GET_CONNECTION = NetworkMessagesActionTypes.NETWORK_GET_CONNECTION;
export const NETWORK_GET_CONNECTION_SUCCESS = NetworkMessagesActionTypes.NETWORK_GET_CONNECTION_SUCCESS;
export const NETWORK_CHANGE_TAB = NetworkMessagesActionTypes.NETWORK_CHANGE_TAB;
export const NETWORK_GET_SPECIFIC_MESSAGE = NetworkMessagesActionTypes.NETWORK_GET_SPECIFIC_MESSAGE;

export interface NetworkMessagesAction extends FeatureAction<NetworkMessagesActionTypes> {
  readonly type: NetworkMessagesActionTypes;
}

export class NetworkMessagesInit implements NetworkMessagesAction {
  readonly type = NETWORK_INIT;
}

export class NetworkMessagesClose implements NetworkMessagesAction {
  readonly type = NETWORK_CLOSE;
}

export class NetworkMessagesGetMessages implements NetworkMessagesAction {
  readonly type = NETWORK_GET_MESSAGES;
}

export class NetworkMessagesGetPaginatedMessages implements NetworkMessagesAction {
  readonly type = NETWORK_GET_PAGINATED_MESSAGES;

  constructor(public payload: { id?: number, direction?: NetworkMessagesDirection, timestamp?: TimestampInterval }) {}
}

export class NetworkMessagesGetMessagesSuccess implements NetworkMessagesAction {
  readonly type = NETWORK_GET_MESSAGES_SUCCESS;

  constructor(public payload: NetworkMessage[]) {}
}

export class NetworkMessagesToggleFilter implements NetworkMessagesAction {
  readonly type = NETWORK_TOGGLE_FILTER;

  constructor(public payload: { filters: NetworkMessagesFilter[], type: 'add' | 'remove', timestamp?: TimestampInterval, direction?: NetworkMessagesDirection }) {}
}

export class NetworkMessagesSetTimestampInterval implements NetworkMessagesAction {
  readonly type = NETWORK_SET_TIMESTAMP_INTERVAL;

  constructor(public payload: { timestamp: TimestampInterval, direction?: NetworkMessagesDirection }) {}
}

export class NetworkMessagesSetActiveRow implements NetworkMessagesAction {
  readonly type = NETWORK_SET_ACTIVE_ROW;

  constructor(public payload: NetworkMessage) {}
}

export class NetworkMessagesGoLive implements NetworkMessagesAction {
  readonly type = NETWORK_GO_LIVE;
}

export class NetworkMessagesPause implements NetworkMessagesAction {
  readonly type = NETWORK_PAUSE;
}

export class NetworkMessagesGetFullMessage implements NetworkMessagesAction {
  readonly type = NETWORK_GET_FULL_MESSAGE;

  constructor(public payload: { id: number }) {}
}

export class NetworkMessagesGetFullMessageSuccess implements NetworkMessagesAction {
  readonly type = NETWORK_GET_FULL_MESSAGE_SUCCESS;

  constructor(public payload: any) {}
}

export class NetworkMessagesGetConnection implements NetworkMessagesAction {
  readonly type = NETWORK_GET_CONNECTION;

  constructor(public payload: { id: number }) {}
}

export class NetworkMessagesGetConnectionSuccess implements NetworkMessagesAction {
  readonly type = NETWORK_GET_CONNECTION_SUCCESS;

  constructor(public payload: NetworkMessageConnection) {}
}

export class NetworkMessagesGetMessageHex implements NetworkMessagesAction {
  readonly type = NETWORK_GET_MESSAGE_HEX;

  constructor(public payload: { id: number }) {}
}

export class NetworkMessagesGetMessageHexSuccess implements NetworkMessagesAction {
  readonly type = NETWORK_GET_MESSAGE_HEX_SUCCESS;

  constructor(public payload: string) {}
}

export class NetworkMessagesChangeTab implements NetworkMessagesAction {
  readonly type = NETWORK_CHANGE_TAB;

  constructor(public payload: number) {}
}

export class NetworkMessagesGetSpecificMessage implements NetworkMessagesAction {
  readonly type = NETWORK_GET_SPECIFIC_MESSAGE;

  constructor(public payload: { id: number, filters: NetworkMessagesFilter[], type: 'add' | 'remove', timestamp: TimestampInterval, direction?: NetworkMessagesDirection }) {}
}

export type NetworkMessagesActions =
  | NetworkMessagesInit
  | NetworkMessagesClose
  | NetworkMessagesGetMessages
  | NetworkMessagesGetMessagesSuccess
  | NetworkMessagesGetPaginatedMessages
  | NetworkMessagesToggleFilter
  | NetworkMessagesSetTimestampInterval
  | NetworkMessagesSetActiveRow
  | NetworkMessagesGoLive
  | NetworkMessagesPause
  | NetworkMessagesGetFullMessage
  | NetworkMessagesGetFullMessageSuccess
  | NetworkMessagesGetMessageHex
  | NetworkMessagesGetMessageHexSuccess
  | NetworkMessagesGetConnection
  | NetworkMessagesGetConnectionSuccess
  | NetworkMessagesChangeTab
  | NetworkMessagesGetSpecificMessage
  ;
