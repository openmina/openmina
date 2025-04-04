import { NgModule } from '@angular/core';
import { NoPreloading, RouterModule, Routes } from '@angular/router';
import { CONFIG, getFirstFeature } from '@shared/constants/config';
import { WebNodeLandingPageComponent } from '@app/layout/web-node-landing-page/web-node-landing-page.component';
import { getMergedRoute, MergedRoute } from '@openmina/shared';
import { filter, take } from 'rxjs';
import { landingPageGuard } from '@shared/guards/landing-page.guard';

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
export const WEBNODE_TITLE: string = APP_TITLE + ' - Web Node';
export const FUZZING_TITLE: string = APP_TITLE + ' - Fuzzing';


function generateRoutes(): Routes {
  const routes: Routes = [
    {
      path: 'dashboard',
      loadChildren: () => import('@dashboard/dashboard.module').then(m => m.DashboardModule),
      title: DASHBOARD_TITLE,
      canActivate: [landingPageGuard],
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
      canActivate: [landingPageGuard],
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
      canActivate: [landingPageGuard],
    },
    {
      path: 'mempool',
      loadChildren: () => import('@mempool/mempool.module').then(m => m.MempoolModule),
      title: MEMPOOL_TITLE,
      canActivate: [landingPageGuard],
    },
    {
      path: 'benchmarks',
      loadChildren: () => import('@benchmarks/benchmarks.module').then(m => m.BenchmarksModule),
      title: BENCHMARKS_TITLE,
      canActivate: [landingPageGuard],
    },
    {
      path: 'fuzzing',
      loadChildren: () => import('@fuzzing/fuzzing.module').then(m => m.FuzzingModule),
      title: FUZZING_TITLE,
    },
    {
      path: 'loading-web-node',
      loadChildren: () => import('@web-node/web-node.module').then(m => m.WebNodeModule),
      title: WEBNODE_TITLE,
      canActivate: [landingPageGuard],
    },
  ];
  if (CONFIG.showLeaderboard) {
    routes.push({
      path: '',
      loadChildren: () => import('@leaderboard/leaderboard.module').then(m => m.LeaderboardModule),
    });
  } else if (CONFIG.showWebNodeLandingPage) {
    routes.push({
      path: '',
      component: WebNodeLandingPageComponent,
    });
  }

  return [
    ...routes,
    {
      path: '**',
      redirectTo: CONFIG.showWebNodeLandingPage ? '' : getFirstFeature(),
      pathMatch: 'full',
    },
  ];
}


@NgModule({
  imports: [
    RouterModule.forRoot(generateRoutes(), {
      // enableTracing: true,
      preloadingStrategy: NoPreloading,
      onSameUrlNavigation: 'ignore',
      initialNavigation: 'enabledNonBlocking',
    }),
  ],
  exports: [RouterModule],
})
export class AppRouting {}
