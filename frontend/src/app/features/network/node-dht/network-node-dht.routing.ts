import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NetworkNodeDhtComponent } from '@network/node-dht/network-node-dht.component';
import { NETWORK_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    component: NetworkNodeDhtComponent,
    path: '',
    title: NETWORK_TITLE,
    children: [
      {
        path: ':id',
        component: NetworkNodeDhtComponent,
        title: NETWORK_TITLE,
      },
    ],
  },
  {
    path: '**',
    redirectTo: '',
    pathMatch: 'full',
    title: NETWORK_TITLE,
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class NetworkNodeDhtRouting {
}
