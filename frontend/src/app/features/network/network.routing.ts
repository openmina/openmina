import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NetworkComponent } from '@network/network.component';
import { NETWORK_TITLE } from '@app/app.routing';
import { networkGuard } from '@network/network.guard';

const routes: Routes = [
  {
    path: '',
    component: NetworkComponent,
    children: [
      {
        path: 'messages',
        loadChildren: () => import('./messages/network-messages.module').then(m => m.NetworkMessagesModule),
        canActivate: [networkGuard],
        title: NETWORK_TITLE,
      },
      {
        path: 'connections',
        loadChildren: () => import('./connections/network-connections.module').then(m => m.NetworkConnectionsModule),
        canActivate: [networkGuard],
        title: NETWORK_TITLE,
      },
      {
        path: 'blocks',
        loadChildren: () => import('./blocks/network-blocks.module').then(m => m.NetworkBlocksModule),
        canActivate: [networkGuard],
        title: NETWORK_TITLE,
      },
      {
        path: 'topology',
        loadChildren: () => import('./splits/dashboard-splits.module').then(m => m.DashboardSplitsModule),
        canActivate: [networkGuard],
        title: NETWORK_TITLE,
      },
      {
        path: 'node-dht',
        loadChildren: () => import('./node-dht/network-node-dht.module').then(m => m.NetworkNodeDhtModule),
        canActivate: [networkGuard],
        title: NETWORK_TITLE,
      },
      {
        path: 'graph-overview',
        loadChildren: () => import('./dht-graph/dht-graph.module').then(m => m.DhtGraphModule),
        canActivate: [networkGuard],
        title: NETWORK_TITLE,
      },
      {
        path: 'bootstrap-stats',
        loadChildren: () => import('./bootstrap-stats/network-bootstrap-stats.module').then(m => m.NetworkBootstrapStatsModule),
        canActivate: [networkGuard],
        title: NETWORK_TITLE,
      },
      {
        path: '**',
        redirectTo: 'messages',
        pathMatch: 'full',
      },
    ],
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class NetworkRoutingModule {
}
