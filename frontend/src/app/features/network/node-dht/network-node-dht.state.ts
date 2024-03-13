import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectNetworkNodeDhtState } from '@network/network.state';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';

export interface NetworkNodeDhtState {
  peers: NetworkNodeDHT[];
  thisKey: string;
}

const select = <T>(selector: (state: NetworkNodeDhtState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkNodeDhtState,
  selector,
);

export const selectNetworkNodeDhtPeers = select((network: NetworkNodeDhtState): NetworkNodeDHT[] => network.peers);
