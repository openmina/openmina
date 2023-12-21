import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NETWORK_TITLE } from '@app/app.routing';
import { NetworkConnectionsComponent } from '@network/connections/network-connections.component';

const routes: Routes = [
  {
    path: '',
    component: NetworkConnectionsComponent,
    children: [
      {
        path: ':id',
        component: NetworkConnectionsComponent,
        title: NETWORK_TITLE,
      },
    ],
  },
  {
    path: '**',
    redirectTo: '',
    pathMatch: 'full',
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class NetworkConnectionsRouting {}
