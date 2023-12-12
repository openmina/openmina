import { NetworkBlock } from '@shared/types/network/blocks/network-block.type';
import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectNetworkBlocksState } from '@network/network.state';
import { TableSort } from '@openmina/shared';

export interface NetworkBlocksState {
  blocks: NetworkBlock[];
  filteredBlocks: NetworkBlock[];
  stream: boolean;
  sort: TableSort<NetworkBlock>;
  openSidePanel: boolean;
  allFilters: string[];
  activeFilters: string[];
  activeBlock: number;
  earliestBlock: number;
}

const select = <T>(selector: (state: NetworkBlocksState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkBlocksState,
  selector,
);

export const selectNetworkBlocks = select((network: NetworkBlocksState): NetworkBlock[] => network.filteredBlocks);
export const selectNetworkBlocksActiveBlock = select((network: NetworkBlocksState): number => network.activeBlock);
export const selectNetworkBlocksEarliestBlock = select((network: NetworkBlocksState): number => network.earliestBlock);
export const selectNetworkBlocksSorting = select((network: NetworkBlocksState): TableSort<NetworkBlock> => network.sort);
export const selectNetworkBlocksSidePanelOpen = select((network: NetworkBlocksState): boolean => network.openSidePanel);
export const selectNetworkBlocksAllFilters = select((network: NetworkBlocksState): string[] => network.allFilters);
export const selectNetworkBlocksActiveFilters = select((network: NetworkBlocksState): string[] => network.activeFilters);
