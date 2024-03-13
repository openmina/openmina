import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectNetworkNodeDhtState } from '@network/network.state';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBootstrapStats } from '@shared/types/network/node-dht/network-node-dht-bootstrap-stats.type';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';

export interface NetworkNodeDhtState {
  peers: NetworkNodeDhtPeer[];
  thisKey: string;
  activePeer: NetworkNodeDhtPeer;
  openSidePanel: boolean;
  boostrapStats: NetworkNodeDhtBootstrapStats[];
  activeBootstrapRequest: any;
  buckets: NetworkNodeDhtBucket[];
}

const select = <T>(selector: (state: NetworkNodeDhtState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkNodeDhtState,
  selector,
);

export const selectNetworkNodeDhtPeers = select((network: NetworkNodeDhtState): NetworkNodeDhtPeer[] => network.peers);
export const selectNetworkNodeDhtActivePeer = select((network: NetworkNodeDhtState): NetworkNodeDhtPeer => network.activePeer);
export const selectNetworkNodeDhtOpenSidePanel = select((network: NetworkNodeDhtState): boolean => network.openSidePanel);
export const selectNetworkNodeDhtBootstrapStats = select((network: NetworkNodeDhtState): NetworkNodeDhtBootstrapStats[] => network.boostrapStats);
export const selectNetworkNodeDhtActiveBootstrapRequest = select((network: NetworkNodeDhtState): any => network.activeBootstrapRequest);
export const selectNetworkNodeDhtBuckets = select((network: NetworkNodeDhtState): NetworkNodeDhtBucket[] => network.buckets);
