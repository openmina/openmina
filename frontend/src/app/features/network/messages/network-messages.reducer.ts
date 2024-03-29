import { NetworkMessagesState } from '@network/messages/network-messages.state';
import {
  NETWORK_CHANGE_TAB,
  NETWORK_CLOSE,
  NETWORK_GET_CONNECTION_SUCCESS,
  NETWORK_GET_FULL_MESSAGE_SUCCESS,
  NETWORK_GET_MESSAGE_HEX_SUCCESS,
  NETWORK_GET_MESSAGES,
  NETWORK_GET_MESSAGES_SUCCESS,
  NETWORK_GET_PAGINATED_MESSAGES,
  NETWORK_GET_SPECIFIC_MESSAGE,
  NETWORK_GO_LIVE,
  NETWORK_INIT,
  NETWORK_PAUSE,
  NETWORK_SET_ACTIVE_ROW,
  NETWORK_SET_TIMESTAMP_INTERVAL,
  NETWORK_TOGGLE_FILTER,
  NetworkMessagesActions,
} from '@network/messages/network-messages.actions';
import { NetworkMessage } from '@shared/types/network/messages/network-message.type';
import { ONE_THOUSAND, VirtualScrollActivePage } from '@openmina/shared';
import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';

const initialState: NetworkMessagesState = {
  messages: [],
  activeRow: undefined,
  activeRowFullMessage: undefined,
  activeRowHex: undefined,
  activeFilters: [],
  timestamp: {
    from: undefined,
    to: undefined,
  },
  stream: false,
  connection: undefined,
  limit: ONE_THOUSAND,
  direction: NetworkMessagesDirection.REVERSE,
  activePage: {},
  pages: [],
  activeTab: 1,
};

export function networkMessagesReducer(state: NetworkMessagesState = initialState, action: NetworkMessagesActions): NetworkMessagesState {
  switch (action.type) {

    case NETWORK_GET_MESSAGES: {
      return {
        ...state,
        activePage: {
          ...state.activePage,
          firstPageIdWithFilters: null,
        },
      };
    }

    case NETWORK_GET_MESSAGES_SUCCESS: {
      const activePage = setActivePage(action.payload, state);

      return {
        ...state,
        messages: action.payload,
        activePage,
        pages: setPages(activePage, state),
      };
    }

    case NETWORK_GET_FULL_MESSAGE_SUCCESS: {
      return {
        ...state,
        activeRowFullMessage: action.payload,
      };
    }

    case NETWORK_GET_MESSAGE_HEX_SUCCESS: {
      return {
        ...state,
        activeRowHex: action.payload,
      };
    }

    case NETWORK_GET_CONNECTION_SUCCESS: {
      return {
        ...state,
        connection: action.payload,
      };
    }

    case NETWORK_TOGGLE_FILTER:
    case NETWORK_GET_SPECIFIC_MESSAGE: {
      const activeFilters = action.payload.type === 'add'
        ? [
          ...state.activeFilters,
          ...action.payload.filters.filter(f => !state.activeFilters.some(fi => fi.value === f.value)),
        ]
        : state.activeFilters.filter(f => !action.payload.filters.some(fi => fi.value === f.value));
      return {
        ...state,
        activeFilters,
        stream: false,
        activePage: {
          ...state,
          firstPageIdWithFilters: null,
          lastPageIdWithFilters: null,
          firstPageIdWithTimestamp: null,
          lastPageIdWithTimestamp: -1,
        },
        direction: action.payload.direction ?? (!activeFilters.length && !state.timestamp.from) ? NetworkMessagesDirection.REVERSE : state.direction,
        timestamp: !action.payload.timestamp ? state.timestamp : {
          from: action.payload.timestamp.from,
          to: action.payload.timestamp.to,
        },
        pages: [],
      };
    }

    case NETWORK_SET_ACTIVE_ROW: {
      return {
        ...state,
        activeRow: action.payload,
        activeRowFullMessage: undefined,
        activeRowHex: undefined,
        connection: undefined,
      };
    }

    case NETWORK_INIT:
    case NETWORK_GO_LIVE: {
      return {
        ...state,
        stream: true,
        activePage: {
          ...state.activePage,
          firstPageIdWithFilters: null,
        },
      };
    }

    case NETWORK_PAUSE: {
      return {
        ...state,
        stream: false,
      };
    }

    case NETWORK_GET_PAGINATED_MESSAGES: {
      return {
        ...state,
        stream: false,
        direction: action.payload.direction,
        activePage: {
          ...state.activePage,
          firstPageIdWithFilters: action.payload.id === 0 ? -1 : state.activePage.firstPageIdWithFilters,
          lastPageIdWithTimestamp: action.payload.timestamp?.to && action.payload.timestamp?.from ? null : -1,
        },
      };
    }

    case NETWORK_SET_TIMESTAMP_INTERVAL: {
      return {
        ...state,
        stream: false,
        timestamp: {
          from: action.payload.timestamp.from,
          to: action.payload.timestamp.to,
        },
        direction: action.payload.direction,
        activePage: {
          ...state,
          firstPageIdWithTimestamp: null,
          lastPageIdWithTimestamp: -1,
        },
        pages: [],
      };
    }

    case NETWORK_CHANGE_TAB: {
      return {
        ...state,
        activeTab: action.payload,
      };
    }

    case NETWORK_CLOSE: {
      return initialState;
    }

    default:
      return state;
  }
}

function setActivePage(messages: NetworkMessage[], state: NetworkMessagesState): VirtualScrollActivePage<NetworkMessage> {
  if (!messages.length) {
    return {};
  }
  return {
    id: messages[messages.length - 1].id,
    start: messages[0],
    end: messages[messages.length - 1],
    numberOfRecords: messages.length,
    firstPageIdWithFilters: state.activePage.firstPageIdWithFilters === -1 ? messages[0].id : state.activePage.firstPageIdWithFilters,
    lastPageIdWithFilters: state.activePage.lastPageIdWithFilters === null ? messages[messages.length - 1].id : state.activePage.lastPageIdWithFilters,
    firstPageIdWithTimestamp: state.timestamp.from && !state.activePage.firstPageIdWithTimestamp ? messages[0].id : state.activePage.firstPageIdWithTimestamp,
    lastPageIdWithTimestamp: state.timestamp.from && !state.activePage.lastPageIdWithTimestamp ? messages[messages.length - 1].id : state.activePage.lastPageIdWithTimestamp,
  };
}

function setPages(activePage: VirtualScrollActivePage<NetworkMessage>, state: NetworkMessagesState): number[] {
  const currentPages = state.pages;

  if (currentPages.includes(activePage.id)) {
    return currentPages;
  }

  // if the new page is bigger than the biggest known page, we can reset
  if (activePage.id > currentPages[currentPages.length - 1]) {
    return [activePage.id];
  }

  return [...currentPages, activePage.id].sort((a, b) => a - b);
}
