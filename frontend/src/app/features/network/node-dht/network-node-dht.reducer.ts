import { NetworkNodeDhtState } from '@network/node-dht/network-node-dht.state';
import {
  NETWORK_NODE_DHT_CLOSE,
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS,
  NetworkNodeDhtActions
} from '@network/node-dht/network-node-dht.actions';

const initialState: NetworkNodeDhtState = {
  peers: [],
  thisKey: '',
};

export function networkDhtReducer(state: NetworkNodeDhtState = initialState, action: NetworkNodeDhtActions): NetworkNodeDhtState {
  switch (action.type) {

    case NETWORK_NODE_DHT_GET_PEERS_SUCCESS: {
      return {
        ...state,
        peers: action.payload.peers,
        thisKey: action.payload.thisKey,
      };
    }

    case NETWORK_NODE_DHT_CLOSE:
      return initialState;

    default:
      return state;
  }
}
