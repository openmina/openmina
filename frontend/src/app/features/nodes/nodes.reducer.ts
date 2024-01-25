import { ActionReducer, combineReducers } from '@ngrx/store';

import * as fromDashboard from '@nodes/overview/nodes-overview.reducer';
import { NodesOverviewAction, NodesOverviewActions } from '@nodes/overview/nodes-overview.actions';

import * as fromBootstrap from '@nodes/bootstrap/nodes-bootstrap.reducer';
import { NodesBootstrapAction, NodesBootstrapActions } from '@nodes/bootstrap/nodes-bootstrap.actions';

import * as fromLive from '@nodes/live/nodes-live.reducer';
import { NodesLiveAction, NodesLiveActions } from '@nodes/live/nodes-live.actions';
import { NodesState } from '@nodes/nodes.state';

export type NodesActions =
  & NodesOverviewActions
  & NodesBootstrapActions
  & NodesLiveActions
  ;
export type NodesAction =
  & NodesOverviewAction
  & NodesBootstrapAction
  & NodesLiveAction
  ;

export const nodesReducer: ActionReducer<NodesState, NodesActions> = combineReducers<NodesState, NodesActions>({
  overview: fromDashboard.reducer,
  bootstrap: fromBootstrap.reducer,
  live: fromLive.reducer,
});
