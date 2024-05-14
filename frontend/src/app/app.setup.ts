import { ActionReducerMap, MetaReducer } from '@ngrx/store';

import { errorReducer } from '@error-preview/error-preview.reducer';
import { ErrorPreviewAction } from '@error-preview/error-preview.actions';
import { ErrorPreviewState } from '@error-preview/error-preview.state';

import { appReducer } from '@app/app.reducer';
import { AppAction } from '@app/app.actions';
import { AppState } from '@app/app.state';

import { loadingReducer, LoadingState } from '@app/layout/toolbar/loading.reducer';

import { dashboardReducer } from '@dashboard/dashboard.reducer';
import { DashboardAction } from '@dashboard/dashboard.actions';
import { DashboardState } from '@dashboard/dashboard.state';

import { NetworkAction, networkReducer } from '@network/network.reducer';
import { NetworkState } from '@network/network.state';

import { NodesAction, nodesReducer } from '@nodes/nodes.reducer';
import { NodesState } from '@nodes/nodes.state';

import { StateAction, stateReducer } from '@state/state.reducer';
import { StateState } from '@state/state.state';

import { SnarksAction, snarksReducer } from '@snarks/snarks.reducer';
import { SnarksState } from '@snarks/snarks.state';

import { TestingToolAction, testingToolReducer } from '@testing-tool/testing-tool.reducer';
import { TestingToolState } from '@testing-tool/testing-tool.state';

import { ResourcesAction, resourcesReducer } from '@resources/resources.reducer';
import { ResourcesState } from '@resources/resources.state';

import { blockProductionReducer } from '@block-production/block-production.reducer';
import { BlockProductionState } from '@block-production/block-production.state';
import { BLOCK_PRODUCTION_KEY } from '@block-production/block-production.actions';

export interface MinaState {
  app: AppState;
  [BLOCK_PRODUCTION_KEY]: BlockProductionState;
  dashboard: DashboardState;
  error: ErrorPreviewState;
  loading: LoadingState;
  network: NetworkState;
  nodes: NodesState;
  resources: ResourcesState;
  state: StateState;
  snarks: SnarksState;
  testingTool: TestingToolState;
}

type MinaAction =
  & AppAction
  & ErrorPreviewAction
  & DashboardAction
  & NetworkAction
  & NodesAction
  & ResourcesAction
  & StateAction
  & SnarksAction
  & TestingToolAction
  ;

export const reducers: ActionReducerMap<MinaState, MinaAction> = {
  app: appReducer,
  error: errorReducer,
  loading: loadingReducer,
  [BLOCK_PRODUCTION_KEY]: blockProductionReducer,
  dashboard: dashboardReducer,
  network: networkReducer,
  nodes: nodesReducer,
  resources: resourcesReducer,
  state: stateReducer,
  snarks: snarksReducer,
  testingTool: testingToolReducer,
};

export const metaReducers: MetaReducer<MinaState, MinaAction>[] = [];

export const selectMinaState = (state: MinaState): MinaState => state;
