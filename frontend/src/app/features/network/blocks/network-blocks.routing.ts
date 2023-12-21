import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NetworkBlocksComponent } from '@network/blocks/network-blocks.component';
import { NETWORK_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    path: '',
    component: NetworkBlocksComponent,
    children: [
      {
        path: ':height',
        component: NetworkBlocksComponent,
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
export class NetworkBlocksRouting {}
