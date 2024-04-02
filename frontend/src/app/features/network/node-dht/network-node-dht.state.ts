import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectNetworkNodeDhtState } from '@network/network.state';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';

export interface NetworkNodeDhtState {
  peers: NetworkNodeDhtPeer[];
  thisKey: string;
  activePeer: NetworkNodeDhtPeer;
  buckets: NetworkNodeDhtBucket[];
  sidePanelWidth: number;
}

const select = <T>(selector: (state: NetworkNodeDhtState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkNodeDhtState,
  selector,
);

export const selectNetworkNodeDhtPeers = select((network: NetworkNodeDhtState): NetworkNodeDhtPeer[] => network.peers);
export const selectNetworkNodeDhtActivePeer = select((network: NetworkNodeDhtState): NetworkNodeDhtPeer => network.activePeer);
export const selectNetworkNodeDhtKeyPeersBucketsOpenSidePanel = select((network: NetworkNodeDhtState): [string, NetworkNodeDhtPeer[], NetworkNodeDhtBucket[], boolean] => [
  network.thisKey,
  network.peers,
  network.buckets,
  !!network.activePeer,
]);
