import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';
import { ONE_THOUSAND } from '@openmina/shared';
import { NetworkConnectionsState } from '@network/connections/network-connections.state';
import {
  NETWORK_CONNECTIONS_CLOSE,
  NETWORK_CONNECTIONS_GET_CONNECTIONS_SUCCESS,
  NETWORK_CONNECTIONS_GO_LIVE,
  NETWORK_CONNECTIONS_INIT,
  NETWORK_CONNECTIONS_PAUSE,
  NETWORK_CONNECTIONS_SELECT_CONNECTION,
  NetworkConnectionsActions,
} from '@network/connections/network-connections.actions';

const initialState: NetworkConnectionsState = {
  connections: [],
  activeConnection: undefined,
  stream: false,
  limit: ONE_THOUSAND,
  direction: NetworkMessagesDirection.REVERSE,
};

export function networkConnectionsReducer(state: NetworkConnectionsState = initialState, action: NetworkConnectionsActions): NetworkConnectionsState {
  switch (action.type) {

    case NETWORK_CONNECTIONS_GET_CONNECTIONS_SUCCESS: {
      return {
        ...state,
        connections: action.payload,
      };
    }

    case NETWORK_CONNECTIONS_SELECT_CONNECTION: {
      return {
        ...state,
        activeConnection: action.payload,
      };
    }

    case NETWORK_CONNECTIONS_INIT:
    case NETWORK_CONNECTIONS_GO_LIVE: {
      return {
        ...state,
        stream: true,
      };
    }

    case NETWORK_CONNECTIONS_PAUSE: {
      return {
        ...state,
        stream: false,
      };
    }

    case NETWORK_CONNECTIONS_CLOSE:
      return initialState;

    default:
      return state;
  }
}
