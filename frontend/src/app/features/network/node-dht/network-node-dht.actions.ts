import { FeatureAction } from '@openmina/shared';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';

enum NetworkNodeDhtActionTypes {
  NETWORK_NODE_DHT_INIT = 'NETWORK_NODE_DHT_INIT',
  NETWORK_NODE_DHT_CLOSE = 'NETWORK_NODE_DHT_CLOSE',
  NETWORK_NODE_DHT_GET_PEERS = 'NETWORK_NODE_DHT_GET_PEERS',
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS = 'NETWORK_NODE_DHT_GET_PEERS_SUCCESS',
  NETWORK_NODE_DHT_SET_ACTIVE_PEER = 'NETWORK_NODE_DHT_SET_ACTIVE_PEER',
}

export const NETWORK_NODE_DHT_INIT = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_INIT;
export const NETWORK_NODE_DHT_CLOSE = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_CLOSE;
export const NETWORK_NODE_DHT_GET_PEERS = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_GET_PEERS;
export const NETWORK_NODE_DHT_GET_PEERS_SUCCESS = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_GET_PEERS_SUCCESS;
export const NETWORK_NODE_DHT_SET_ACTIVE_PEER = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_SET_ACTIVE_PEER;

export interface NetworkNodeDhtAction extends FeatureAction<NetworkNodeDhtActionTypes> {
  readonly type: NetworkNodeDhtActionTypes;
}

export class NetworkNodeDhtInit implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_INIT;
}

export class NetworkNodeDhtClose implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_CLOSE;
}

export class NetworkNodeDhtGetPeers implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_GET_PEERS;
}

export class NetworkNodeDhtGetPeersSuccess implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_GET_PEERS_SUCCESS;

  constructor(public payload: { peers: NetworkNodeDHT[], thisKey: string }) { }
}

export class NetworkNodeDhtSetActivePeer implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_SET_ACTIVE_PEER;

  constructor(public payload: NetworkNodeDHT) { }
}


export type NetworkNodeDhtActions =
  | NetworkNodeDhtInit
  | NetworkNodeDhtClose
  | NetworkNodeDhtGetPeers
  | NetworkNodeDhtGetPeersSuccess
  | NetworkNodeDhtSetActivePeer
  ;
