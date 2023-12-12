import { NgModule } from '@angular/core';
import { PreloadAllModules, RouterModule, Routes } from '@angular/router';
import { getFirstFeature } from '@shared/constants/config';

const APP_TITLE: string = 'Open Mina';

export const DASHBOARD_TITLE: string = APP_TITLE + ' - Dashboard';
export const NODES_TITLE: string = APP_TITLE + ' - Nodes';
export const STATE_TITLE: string = APP_TITLE + ' - State';
export const SNARKS_TITLE: string = APP_TITLE + ' - Snarks';
export const TESTING_TOOL_TITLE: string = APP_TITLE + ' - Testing Tool';


const routes: Routes = [
  {
    path: 'dashboard',
    loadChildren: () => import('@dashboard/dashboard.module').then(m => m.DashboardModule),
    title: DASHBOARD_TITLE,
  },
  {
    path: 'nodes',
    loadChildren: () => import('@nodes/nodes.module').then(m => m.NodesModule),
    title: NODES_TITLE,
    // canActivate: [FeatureGuard],
  },
  {
    path: 'state',
    loadChildren: () => import('@state/state.module').then(m => m.StateModule),
    title: STATE_TITLE,
  },
  {
    path: 'snarks',
    loadChildren: () => import('@snarks/snarks.module').then(m => m.SnarksModule),
    title: SNARKS_TITLE,
  },
  {
    path: 'testing-tool',
    loadChildren: () => import('@testing-tool/testing-tool.module').then(m => m.TestingToolModule),
    title: TESTING_TOOL_TITLE,
  },
  {
    path: '**',
    redirectTo: getFirstFeature(),
    pathMatch: 'full',
  },
];

@NgModule({
  imports: [
    RouterModule.forRoot(routes, {
      // enableTracing: true,
      preloadingStrategy: PreloadAllModules,
      onSameUrlNavigation: 'ignore',
      initialNavigation: 'enabledNonBlocking',
    }),
  ],
  exports: [RouterModule],
})
export class AppRouting {}
