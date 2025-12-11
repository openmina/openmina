import { createFeatureSelector, createSelector } from '@ngrx/store';
import { routerStateConfig } from './ngrx-router.module';
import { MergedRoute, MergedRouteReducerState } from './merged-route';

const getRouterReducerState = createFeatureSelector<MergedRouteReducerState>(routerStateConfig.stateKey);

export const getMergedRoute = createSelector(getRouterReducerState, (routerReducerState: MergedRouteReducerState): MergedRoute => routerReducerState?.state);
