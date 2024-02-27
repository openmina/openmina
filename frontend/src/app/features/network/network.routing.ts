import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NetworkComponent } from '@network/network.component';
import { NETWORK_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    path: '',
    component: NetworkComponent,
    children: [
      {
        path: 'messages',
        loadChildren: () => import('./messages/network-messages.module').then(m => m.NetworkMessagesModule),
        title: NETWORK_TITLE,
      },
      {
        path: 'connections',
        loadChildren: () => import('./connections/network-connections.module').then(m => m.NetworkConnectionsModule),
        title: NETWORK_TITLE,
      },
      {
        path: 'blocks',
        loadChildren: () => import('./blocks/network-blocks.module').then(m => m.NetworkBlocksModule),
        title: NETWORK_TITLE,
      },
      {
        path: 'topology',
        loadChildren: () => import('./splits/dashboard-splits.module').then(m => m.DashboardSplitsModule),
        title: NETWORK_TITLE,
      },
      {
        path: 'node-dht',
        loadChildren: () => import('./node-dht/node-dht.module').then(m => m.NodeDhtModule),
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
export class NetworkRoutingModule {}
