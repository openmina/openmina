import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { TableSort } from '@openmina/shared';
import { NodesBootstrapNode } from '@shared/types/nodes/bootstrap/nodes-bootstrap-node.type';
import { selectNodesBootstrapState } from '@nodes/nodes.state';

export interface NodesBootstrapState {
  nodes: NodesBootstrapNode[];
  activeNode: NodesBootstrapNode;
  sort: TableSort<NodesBootstrapNode>;
  openSidePanel: boolean;
}

const select = <T>(selector: (state: NodesBootstrapState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNodesBootstrapState,
  selector,
);

export const selectNodesBootstrapNodes = select((state: NodesBootstrapState): NodesBootstrapNode[] => state.nodes);
export const selectNodesBootstrapSort = select((state: NodesBootstrapState): TableSort<NodesBootstrapNode> => state.sort);
export const selectNodesBootstrapActiveNode = select((state: NodesBootstrapState): NodesBootstrapNode => state.activeNode);
export const selectNodesBootstrapOpenSidePanel = select((state: NodesBootstrapState): boolean => state.openSidePanel);
