import { FeatureAction } from '@openmina/shared';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBootstrapStats } from '@shared/types/network/node-dht/network-node-dht-bootstrap-stats.type';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';

enum NetworkNodeDhtActionTypes {
  NETWORK_NODE_DHT_INIT = 'NETWORK_NODE_DHT_INIT',
  NETWORK_NODE_DHT_CLOSE = 'NETWORK_NODE_DHT_CLOSE',
  NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS = 'NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS',
  NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS = 'NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS',
  NETWORK_NODE_DHT_SET_ACTIVE_BOOTSTRAP_REQUEST = 'NETWORK_NODE_DHT_SET_ACTIVE_BOOTSTRAP_REQUEST',
  NETWORK_NODE_DHT_GET_PEERS = 'NETWORK_NODE_DHT_GET_PEERS',
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS = 'NETWORK_NODE_DHT_GET_PEERS_SUCCESS',
  NETWORK_NODE_DHT_SET_ACTIVE_PEER = 'NETWORK_NODE_DHT_SET_ACTIVE_PEER',
  NETWORK_NODE_DHT_TOGGLE_SIDE_PANEL = 'NETWORK_NODE_DHT_TOGGLE_SIDE_PANEL',
}

export const NETWORK_NODE_DHT_INIT = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_INIT;
export const NETWORK_NODE_DHT_CLOSE = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_CLOSE;
export const NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS;
export const NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS;
export const NETWORK_NODE_DHT_SET_ACTIVE_BOOTSTRAP_REQUEST = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_SET_ACTIVE_BOOTSTRAP_REQUEST;
export const NETWORK_NODE_DHT_GET_PEERS = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_GET_PEERS;
export const NETWORK_NODE_DHT_GET_PEERS_SUCCESS = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_GET_PEERS_SUCCESS;
export const NETWORK_NODE_DHT_SET_ACTIVE_PEER = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_SET_ACTIVE_PEER;
export const NETWORK_NODE_DHT_TOGGLE_SIDE_PANEL = NetworkNodeDhtActionTypes.NETWORK_NODE_DHT_TOGGLE_SIDE_PANEL;

export interface NetworkNodeDhtAction extends FeatureAction<NetworkNodeDhtActionTypes> {
  readonly type: NetworkNodeDhtActionTypes;
}

export class NetworkNodeDhtInit implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_INIT;
}

export class NetworkNodeDhtClose implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_CLOSE;
}

export class NetworkNodeDhtGetBootstrapStats implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS;
}

export class NetworkNodeDhtGetBootstrapStatsSuccess implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS;

  constructor(public payload: NetworkNodeDhtBootstrapStats[]) { }
}

export class NetworkNodeDhtSetActiveBootstrapRequest implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_SET_ACTIVE_BOOTSTRAP_REQUEST;

  constructor(public payload: any) { }
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

export class NetworkNodeDhtToggleSidePanel implements NetworkNodeDhtAction {
  readonly type = NETWORK_NODE_DHT_TOGGLE_SIDE_PANEL;
}

export type NetworkNodeDhtActions =
  | NetworkNodeDhtInit
  | NetworkNodeDhtClose
  | NetworkNodeDhtGetBootstrapStats
  | NetworkNodeDhtGetBootstrapStatsSuccess
  | NetworkNodeDhtSetActiveBootstrapRequest
  | NetworkNodeDhtGetPeers
  | NetworkNodeDhtGetPeersSuccess
  | NetworkNodeDhtSetActivePeer
  | NetworkNodeDhtToggleSidePanel
  ;
