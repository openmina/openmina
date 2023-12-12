import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NodesLiveNode } from '@shared/types/nodes/live/nodes-live-node.type';
import { NodesLiveBlockEvent } from '@shared/types/nodes/live/nodes-live-block-event.type';
import { TableSort } from '@openmina/shared';
import { selectNodesLiveState } from '@nodes/nodes.state';

export interface NodesLiveState {
  nodes: NodesLiveNode[];
  activeNode: NodesLiveNode;
  sort: TableSort<NodesLiveBlockEvent>;
  openSidePanel: boolean;
  filteredEvents: NodesLiveBlockEvent[];
  filters: string[];
}

const select = <T>(selector: (state: NodesLiveState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNodesLiveState,
  selector,
);

export const selectNodesLiveNodes = select((state: NodesLiveState): NodesLiveNode[] => state.nodes);
export const selectNodesLiveSort = select((state: NodesLiveState): TableSort<NodesLiveBlockEvent> => state.sort);
export const selectNodesLiveActiveNode = select((state: NodesLiveState): NodesLiveNode => state.activeNode);
export const selectNodesLiveOpenSidePanel = select((state: NodesLiveState): boolean => state.openSidePanel);
export const selectNodesLiveFilters = select((state: NodesLiveState): string[] => state.filters);
export const selectNodesLiveFilteredEvents = select((state: NodesLiveState): NodesLiveBlockEvent[] => state.filteredEvents);
