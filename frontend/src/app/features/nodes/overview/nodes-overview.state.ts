import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { TableSort } from '@openmina/shared';
import { selectNodesDashboardState } from '@nodes/nodes.state';

export interface NodesOverviewState {
  nodes: NodesOverviewNode[];
  activeNode: NodesOverviewNode;
  sort: TableSort<NodesOverviewNode>;
}

const select = <T>(selector: (state: NodesOverviewState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNodesDashboardState,
  selector,
);

export const selectNodesOverviewNodes = select((state: NodesOverviewState): NodesOverviewNode[] => state.nodes);
export const selectNodesOverviewSort = select((state: NodesOverviewState): TableSort<NodesOverviewNode> => state.sort);
export const selectNodesOverviewActiveNode = select((state: NodesOverviewState): NodesOverviewNode => state.activeNode);
