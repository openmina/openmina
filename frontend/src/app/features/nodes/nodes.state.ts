import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { NodesOverviewState } from '@nodes/overview/nodes-overview.state';
import { NodesBootstrapState } from '@nodes/bootstrap/nodes-bootstrap.state';
import { NodesLiveState } from '@nodes/live/nodes-live.state';

export interface NodesState {
  overview: NodesOverviewState;
  bootstrap: NodesBootstrapState;
  live: NodesLiveState;
}

const select = <T>(selector: (state: NodesState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNodesState,
  selector,
);

export const selectNodesState = createFeatureSelector<NodesState>('nodes');
export const selectNodesDashboardState = select((state: NodesState): NodesOverviewState => state.overview);
export const selectNodesBootstrapState = select((state: NodesState): NodesBootstrapState => state.bootstrap);
export const selectNodesLiveState = select((state: NodesState): NodesLiveState => state.live);
