import { NetworkNodeDhtState } from '@network/node-dht/network-node-dht.state';
import {
  NETWORK_NODE_DHT_CLOSE,
  NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS,
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS,
  NETWORK_NODE_DHT_SET_ACTIVE_BOOTSTRAP_REQUEST,
  NETWORK_NODE_DHT_SET_ACTIVE_PEER,
  NETWORK_NODE_DHT_SIDE_PANEL_RESIZE,
  NETWORK_NODE_DHT_TOGGLE_SIDE_PANEL,
  NetworkNodeDhtActions,
} from '@network/node-dht/network-node-dht.actions';

const initialState: NetworkNodeDhtState = {
  peers: [],
  thisKey: '',
  activePeer: undefined,
  openSidePanel: true,
  boostrapStats: undefined,
  activeBootstrapRequest: undefined,
  buckets: [],
  sidePanelWidth: -1,
};

export function networkDhtReducer(state: NetworkNodeDhtState = initialState, action: NetworkNodeDhtActions): NetworkNodeDhtState {
  switch (action.type) {

    case NETWORK_NODE_DHT_GET_PEERS_SUCCESS: {
      if (sameRecord(state, action.payload)) {
        return state;
      }
      return {
        ...state,
        peers: action.payload.peers,
        thisKey: action.payload.thisKey,
        buckets: action.payload.buckets,
      };
    }

    case NETWORK_NODE_DHT_SET_ACTIVE_PEER: {
      return {
        ...state,
        activePeer: action.payload,
      };
    }

    case NETWORK_NODE_DHT_TOGGLE_SIDE_PANEL: {
      return {
        ...state,
        openSidePanel: !state.openSidePanel,
      };
    }

    case NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS: {
      return {
        ...state,
        boostrapStats: action.payload,
      };
    }

    case NETWORK_NODE_DHT_SET_ACTIVE_BOOTSTRAP_REQUEST: {
      return {
        ...state,
        activeBootstrapRequest: action.payload,
      };
    }

    case NETWORK_NODE_DHT_SIDE_PANEL_RESIZE: {
      return {
        ...state,
        sidePanelWidth: action.payload,
      };
    }

    case NETWORK_NODE_DHT_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function sameRecord(state: NetworkNodeDhtState, payload: { peers: any[], thisKey: string }): boolean {
  return state.peers.length === payload.peers.length
    && state.thisKey === payload.thisKey
    && state.peers.every((peer, index) =>
      peer.peerId === payload.peers[index].peerId
      && peer.addressesLength === payload.peers[index].addressesLength
      && peer.addrs.every((addr, addrIndex) => addr === payload.peers[index].addrs[addrIndex])
      && peer.key === payload.peers[index].key
      && peer.hexDistance === payload.peers[index].hexDistance
      && peer.libp2p === payload.peers[index].libp2p,
    );
}
