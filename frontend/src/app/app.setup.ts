import { ActionReducerMap, MetaReducer } from '@ngrx/store';

import { errorReducer } from '@error-preview/error-preview.reducer';
import { ErrorPreviewAction } from '@error-preview/error-preview.actions';
import { ErrorPreviewState } from '@error-preview/error-preview.state';

import { appReducer } from '@app/app.reducer';
import { APP_KEY } from '@app/app.actions';
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

import { ResourcesAction, resourcesReducer } from '@resources/resources.reducer';
import { ResourcesState } from '@resources/resources.state';

import { blockProductionReducer } from '@block-production/block-production.reducer';
import { BlockProductionState } from '@block-production/block-production.state';
import { MempoolState } from '@app/features/mempool/mempool.state';
import { mempoolReducer } from '@app/features/mempool/mempool.reducer';
import { BenchmarksState } from '@benchmarks/benchmarks.state';
import { benchmarksReducer } from '@benchmarks/benchmarks.reducer';
import { fuzzingReducer } from '@fuzzing/fuzzing.reducer';
import { FuzzingState } from '@fuzzing/fuzzing.state';
import { FuzzingAction } from '@fuzzing/fuzzing.actions';
import { LeaderboardState } from '@leaderboard/leaderboard.state';
import { leaderboardReducer } from '@leaderboard/leaderboard.reducer';

export interface MinaState {
  [APP_KEY]: AppState;
  blockProduction: BlockProductionState;
  dashboard: DashboardState;
  error: ErrorPreviewState;
  loading: LoadingState;
  mempool: MempoolState;
  network: NetworkState;
  nodes: NodesState;
  resources: ResourcesState;
  state: StateState;
  snarks: SnarksState;
  benchmarks: BenchmarksState;
  fuzzing: FuzzingState;
  leaderboard: LeaderboardState;
}

type MinaAction =
  & ErrorPreviewAction
  & DashboardAction
  & NetworkAction
  & NodesAction
  & ResourcesAction
  & StateAction
  & SnarksAction
  & FuzzingAction
  ;

export const reducers: ActionReducerMap<MinaState, MinaAction> = {
  [APP_KEY]: appReducer,
  error: errorReducer,
  loading: loadingReducer,
  blockProduction: blockProductionReducer,
  dashboard: dashboardReducer,
  mempool: mempoolReducer,
  network: networkReducer,
  nodes: nodesReducer,
  resources: resourcesReducer,
  state: stateReducer,
  snarks: snarksReducer,
  benchmarks: benchmarksReducer,
  fuzzing: fuzzingReducer,
  leaderboard: leaderboardReducer,
};

export const metaReducers: MetaReducer<MinaState, MinaAction>[] = [];

export const selectMinaState = (state: MinaState): MinaState => state;
