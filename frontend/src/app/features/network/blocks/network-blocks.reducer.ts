import { NetworkBlocksState } from '@network/blocks/network-blocks.state';
import {
  NETWORK_BLOCKS_CLOSE,
  NETWORK_BLOCKS_GET_BLOCKS_SUCCESS,
  NETWORK_BLOCKS_SET_ACTIVE_BLOCK,
  NETWORK_BLOCKS_SET_EARLIEST_BLOCK,
  NETWORK_BLOCKS_SORT,
  NETWORK_BLOCKS_TOGGLE_FILTER,
  NETWORK_BLOCKS_TOGGLE_SIDE_PANEL,
  NetworkBlocksActions,
} from '@network/blocks/network-blocks.actions';
import { ONE_BILLION, sort, SortDirection, TableSort } from '@openmina/shared';
import { NetworkBlock } from '@shared/types/network/blocks/network-block.type';

const initialState: NetworkBlocksState = {
  blocks: [],
  filteredBlocks: [],
  stream: true,
  activeBlock: undefined,
  earliestBlock: undefined,
  sort: {
    sortBy: 'date',
    sortDirection: SortDirection.ASC,
  },
  openSidePanel: false,
  allFilters: [],
  activeFilters: [],
};

export function networkBlocksReducer(state: NetworkBlocksState = initialState, action: NetworkBlocksActions): NetworkBlocksState {

  switch (action.type) {

    case NETWORK_BLOCKS_GET_BLOCKS_SUCCESS: {
      const blocks = sortBlocks(action.payload, state.sort);
      let filteredBlocks = getFilteredBlocks(blocks, state.activeFilters);
      if (state.activeFilters.length > 0) {
        filteredBlocks = applyNewLatencies(filteredBlocks);
      }
      return {
        ...state,
        blocks,
        filteredBlocks,
        allFilters: Array.from(new Set(action.payload.map(b => b.hash))),
      };
    }

    case NETWORK_BLOCKS_SORT: {
      return {
        ...state,
        filteredBlocks: sortBlocks(state.filteredBlocks, action.payload),
        sort: { ...action.payload },
      };
    }

    case NETWORK_BLOCKS_TOGGLE_SIDE_PANEL: {
      return {
        ...state,
        openSidePanel: !state.openSidePanel,
      };
    }

    case NETWORK_BLOCKS_TOGGLE_FILTER: {
      const activeFilters = state.activeFilters.includes(action.payload)
        ? state.activeFilters.filter(f => f !== action.payload)
        : [...state.activeFilters, action.payload];
      const filteredBlocks = getFilteredBlocks(state.blocks, activeFilters);
      return {
        ...state,
        activeFilters,
        filteredBlocks: applyNewLatencies(sortBlocks(filteredBlocks, state.sort)),
      };
    }

    case NETWORK_BLOCKS_SET_ACTIVE_BLOCK: {
      return {
        ...state,
        activeBlock: action.payload.height,
        activeFilters: action.payload.fetchNew ? [] : state.activeFilters,
      };
    }

    case NETWORK_BLOCKS_SET_EARLIEST_BLOCK: {
      return {
        ...state,
        earliestBlock: action.payload.height,
      };
    }

    case NETWORK_BLOCKS_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function getFilteredBlocks(allBlocks: NetworkBlock[], activeFilters: string[]): NetworkBlock[] {
  return activeFilters.length > 0 ? allBlocks.filter(b => activeFilters.includes(b.hash)) : allBlocks;
}

function sortBlocks(blocks: NetworkBlock[], tableSort: TableSort<NetworkBlock>): NetworkBlock[] {
  return sort<NetworkBlock>(blocks, tableSort, ['date', 'hash', 'sender', 'receiver', 'messageKind']);
}

function applyNewLatencies(blocks: NetworkBlock[]): NetworkBlock[] {
  const fastestTime = blocks.map((b: NetworkBlock) => BigInt(b.timestamp)).reduce((t1: bigint, t2: bigint) => t2 < t1 ? t2 : t1);
  return blocks.map(b => ({
    ...b,
    receivedLatency: b.receivedLatency !== undefined ? Number(BigInt(b.timestamp) - fastestTime) / ONE_BILLION : undefined,
    sentLatency: b.sentLatency !== undefined ? Number(BigInt(b.timestamp) - fastestTime) / ONE_BILLION : undefined,
  }));
}
