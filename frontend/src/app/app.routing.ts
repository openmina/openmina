import { NgModule } from '@angular/core';
import { NoPreloading, PreloadAllModules, RouterModule, Routes } from '@angular/router';
import { CONFIG, getFirstFeature } from '@shared/constants/config';

const APP_TITLE: string = 'Open Mina';

export const DASHBOARD_TITLE: string = APP_TITLE + ' - Dashboard';
export const RESOURCES_TITLE: string = APP_TITLE + ' - Resources';
export const NETWORK_TITLE: string = APP_TITLE + ' - Network';
export const NODES_TITLE: string = APP_TITLE + ' - Nodes';
export const STATE_TITLE: string = APP_TITLE + ' - State';
export const SNARKS_TITLE: string = APP_TITLE + ' - Snarks';
export const BLOCK_PRODUCTION_TITLE: string = APP_TITLE + ' - Block Production';
export const MEMPOOL_TITLE: string = APP_TITLE + ' - Mempool';
export const BENCHMARKS_TITLE: string = APP_TITLE + ' - Benchmarks';


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
    path: 'resources',
    loadChildren: () => import('@resources/resources.module').then(m => m.ResourcesModule),
    title: RESOURCES_TITLE,
  },
  {
    path: 'network',
    loadChildren: () => import('@network/network.module').then(m => m.NetworkModule),
    title: NETWORK_TITLE,
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
    path: 'block-production',
    loadChildren: () => import('@block-production/block-production.module').then(m => m.BlockProductionModule),
    title: BLOCK_PRODUCTION_TITLE,
  },
  {
    path: 'mempool',
    loadChildren: () => import('./features/mempool/mempool.module').then(m => m.MempoolModule),
    title: MEMPOOL_TITLE,
  },
  {
    path: 'benchmarks',
    loadChildren: () => import('./features/benchmarks/benchmarks.module').then(m => m.BenchmarksModule),
    title: BENCHMARKS_TITLE,
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
      preloadingStrategy: CONFIG.configs.some(c => c.isWebNode) ? NoPreloading : PreloadAllModules,
      onSameUrlNavigation: 'ignore',
      initialNavigation: 'enabledNonBlocking',
    }),
  ],
  exports: [RouterModule],
})
export class AppRouting {}
