import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NetworkBootstrapStatsComponent } from '@network/bootstrap-stats/network-bootstrap-stats.component';
import { NETWORK_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    component: NetworkBootstrapStatsComponent,
    path: '',
    title: NETWORK_TITLE,
    children: [
      {
        path: ':id',
        component: NetworkBootstrapStatsComponent,
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
export class NetworkBootstrapStatsRouting {}
