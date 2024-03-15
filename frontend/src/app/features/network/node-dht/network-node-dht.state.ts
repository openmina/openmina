import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectNetworkNodeDhtState } from '@network/network.state';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBootstrapStats } from '@shared/types/network/node-dht/network-node-dht-bootstrap-stats.type';

export interface NetworkNodeDhtState {
  peers: NetworkNodeDHT[];
  thisKey: string;
  activePeer: NetworkNodeDHT;
  openSidePanel: boolean;
  boostrapStats: NetworkNodeDhtBootstrapStats;
  activeBootstrapRequest: any;
}

const select = <T>(selector: (state: NetworkNodeDhtState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkNodeDhtState,
  selector,
);

export const selectNetworkNodeDhtPeers = select((network: NetworkNodeDhtState): NetworkNodeDHT[] => network.peers);
export const selectNetworkNodeDhtActivePeer = select((network: NetworkNodeDhtState): NetworkNodeDHT => network.activePeer);
export const selectNetworkNodeDhtOpenSidePanel = select((network: NetworkNodeDhtState): boolean => network.openSidePanel);
export const selectNetworkNodeDhtBootstrapStats = select((network: NetworkNodeDhtState): NetworkNodeDhtBootstrapStats => network.boostrapStats);
export const selectNetworkNodeDhtActiveBootstrapRequest = select((network: NetworkNodeDhtState): any => network.activeBootstrapRequest);
