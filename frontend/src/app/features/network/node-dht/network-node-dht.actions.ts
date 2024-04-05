import { FeatureAction } from '@openmina/shared';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';

enum NetworkNodeDhtActionTypes {
  NETWORK_NODE_DHT_INIT = 'NETWORK_NODE_DHT_INIT',
  NETWORK_NODE_DHT_CLOSE = 'NETWORK_NODE_DHT_CLOSE',
  NETWORK_NODE_DHT_GET_PEERS = 'NETWORK_NODE_DHT_GET_PEERS',
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS = 'NETWORK_NODE_DHT_GET_PEERS_SUCCESS',
  NETWORK_NODE_DHT_SET_ACTIVE_PEER = 'NETWORK_NODE_DHT_SET_ACTIVE_PEER',
  NETWORK_NODE_DHT_SIDE_PANEL_RESIZE = 'NETWORK_NODE_DHT_SIDE_PANEL_RESIZE',
}

export const NETWORK_NODE_DHT_INIT = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_INIT;
export const NETWORK_NODE_DHT_CLOSE = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_CLOSE;
export const NETWORK_NODE_DHT_GET_PEERS = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_GET_PEERS;
export const NETWORK_NODE_DHT_GET_PEERS_SUCCESS = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_GET_PEERS_SUCCESS;
export const NETWORK_NODE_DHT_SET_ACTIVE_PEER = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_SET_ACTIVE_PEER;
export const NETWORK_NODE_DHT_SIDE_PANEL_RESIZE = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_SIDE_PANEL_RESIZE;

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

  constructor(public payload: { peers: NetworkNodeDhtPeer[], thisKey: string, buckets: NetworkNodeDhtBucket[] }) { }
}

export class NetworkNodeDhtSetActivePeer implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_SET_ACTIVE_PEER;

  constructor(public payload: NetworkNodeDhtPeer) { }
}

export class NetworkNodeDhtSidePanelResize implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_SIDE_PANEL_RESIZE;

  constructor(public payload: number) { }
}

export type NetworkNodeDhtActions =
  | NetworkNodeDhtInit
  | NetworkNodeDhtClose
  | NetworkNodeDhtGetPeers
  | NetworkNodeDhtGetPeersSuccess
  | NetworkNodeDhtSetActivePeer
  | NetworkNodeDhtSidePanelResize
  ;
