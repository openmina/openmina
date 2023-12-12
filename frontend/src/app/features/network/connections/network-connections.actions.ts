import { FeatureAction } from '@openmina/shared';
import { NetworkConnection } from '@shared/types/network/connections/network-connection.type';

enum NetworkConnectionsActionTypes {
  NETWORK_CONNECTIONS_INIT = 'NETWORK_CONNECTIONS_INIT',
  NETWORK_CONNECTIONS_CLOSE = 'NETWORK_CONNECTIONS_CLOSE',
  NETWORK_CONNECTIONS_GET_CONNECTIONS = 'NETWORK_CONNECTIONS_GET_CONNECTIONS',
  NETWORK_CONNECTIONS_GET_CONNECTIONS_SUCCESS = 'NETWORK_CONNECTIONS_GET_CONNECTIONS_SUCCESS',
  NETWORK_CONNECTIONS_SELECT_CONNECTION = 'NETWORK_CONNECTIONS_SELECT_CONNECTION',
  NETWORK_CONNECTIONS_GO_LIVE = 'NETWORK_CONNECTIONS_GO_LIVE',
  NETWORK_CONNECTIONS_PAUSE = 'NETWORK_CONNECTIONS_PAUSE',
}

export const NETWORK_CONNECTIONS_INIT = NetworkConnectionsActionTypes.NETWORK_CONNECTIONS_INIT;
export const NETWORK_CONNECTIONS_CLOSE = NetworkConnectionsActionTypes.NETWORK_CONNECTIONS_CLOSE;
export const NETWORK_CONNECTIONS_GET_CONNECTIONS = NetworkConnectionsActionTypes.NETWORK_CONNECTIONS_GET_CONNECTIONS;
export const NETWORK_CONNECTIONS_GET_CONNECTIONS_SUCCESS = NetworkConnectionsActionTypes.NETWORK_CONNECTIONS_GET_CONNECTIONS_SUCCESS;
export const NETWORK_CONNECTIONS_SELECT_CONNECTION = NetworkConnectionsActionTypes.NETWORK_CONNECTIONS_SELECT_CONNECTION;
export const NETWORK_CONNECTIONS_GO_LIVE = NetworkConnectionsActionTypes.NETWORK_CONNECTIONS_GO_LIVE;
export const NETWORK_CONNECTIONS_PAUSE = NetworkConnectionsActionTypes.NETWORK_CONNECTIONS_PAUSE;

export interface NetworkConnectionsAction extends FeatureAction<NetworkConnectionsActionTypes> {
  readonly type: NetworkConnectionsActionTypes;
}

export class NetworkConnectionsInit implements NetworkConnectionsAction {
  readonly type = NETWORK_CONNECTIONS_INIT;
}

export class NetworkConnectionsClose implements NetworkConnectionsAction {
  readonly type = NETWORK_CONNECTIONS_CLOSE;
}

export class NetworkConnectionsGetConnections implements NetworkConnectionsAction {
  readonly type = NETWORK_CONNECTIONS_GET_CONNECTIONS;
}

export class NetworkConnectionsGetConnectionsSuccess implements NetworkConnectionsAction {
  readonly type = NETWORK_CONNECTIONS_GET_CONNECTIONS_SUCCESS;

  constructor(public payload: NetworkConnection[]) {}
}

export class NetworkConnectionsSelectConnection implements NetworkConnectionsAction {
  readonly type = NETWORK_CONNECTIONS_SELECT_CONNECTION;

  constructor(public payload: NetworkConnection) {}
}

export class NetworkConnectionsGoLive implements NetworkConnectionsAction {
  readonly type = NETWORK_CONNECTIONS_GO_LIVE;
}

export class NetworkConnectionsPause implements NetworkConnectionsAction {
  readonly type = NETWORK_CONNECTIONS_PAUSE;
}

export type NetworkConnectionsActions =
  | NetworkConnectionsInit
  | NetworkConnectionsClose
  | NetworkConnectionsGetConnections
  | NetworkConnectionsGetConnectionsSuccess
  | NetworkConnectionsSelectConnection
  | NetworkConnectionsGoLive
  | NetworkConnectionsPause
  ;
