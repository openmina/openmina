import { ActionReducerMap, MetaReducer } from '@ngrx/store';

import * as fromErrorPreview from '@error-preview/error-preview.reducer';
import { ErrorPreviewAction } from '@error-preview/error-preview.actions';
import { ErrorPreviewState } from '@error-preview/error-preview.state';

import * as fromApp from '@app/app.reducer';
import { AppAction } from '@app/app.actions';
import { AppState } from '@app/app.state';

import * as fromLoading from '@app/layout/toolbar/loading.reducer';
import { LoadingState } from '@app/layout/toolbar/loading.reducer';

import * as fromDashboard from '@dashboard/dashboard.reducer';
import { DashboardAction } from '@dashboard/dashboard.actions';
import { DashboardState } from '@dashboard/dashboard.state';

import * as fromNodes from '@nodes/nodes.reducer';
import { NodesAction } from '@nodes/nodes.reducer';
import { NodesState } from '@nodes/nodes.state';

import * as fromState from '@state/state.reducer';
import { StateAction } from '@state/state.reducer';
import { StateState } from '@state/state.state';

import * as fromSnarks from '@snarks/snarks.reducer';
import { SnarksAction } from '@snarks/snarks.reducer';
import { SnarksState } from '@snarks/snarks.state';

import * as fromTestingTool from '@testing-tool/testing-tool.reducer';
import { TestingToolAction } from '@testing-tool/testing-tool.reducer';
import { TestingToolState } from '@testing-tool/testing-tool.state';


export interface MinaState {
  app: AppState;
  error: ErrorPreviewState;
  loading: LoadingState;
  dashboard: DashboardState;
  nodes: NodesState;
  state: StateState;
  snarks: SnarksState;
  testingTool: TestingToolState;
}

type MinaAction =
  & AppAction
  & ErrorPreviewAction
  & StateAction
  & DashboardAction
  & NodesAction
  & SnarksAction
  & TestingToolAction
  ;

export const reducers: ActionReducerMap<MinaState, MinaAction> = {
  app: fromApp.reducer,
  error: fromErrorPreview.reducer,
  loading: fromLoading.reducer,
  dashboard: fromDashboard.reducer,
  nodes: fromNodes.reducer,
  state: fromState.reducer,
  snarks: fromSnarks.reducer,
  testingTool: fromTestingTool.reducer,
};

export const metaReducers: MetaReducer<MinaState, MinaAction>[] = [];

export const selectMinaState = (state: MinaState): MinaState => state;
