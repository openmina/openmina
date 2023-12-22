import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NODES_TITLE } from '@app/app.routing';
import { NodesComponent } from '@nodes/nodes.component';

const routes: Routes = [
  {
    path: '',
    component: NodesComponent,
    children: [
      {
        path: 'overview',
        loadChildren: () => import('@nodes/overview/nodes-overview.module').then(m => m.NodesOverviewModule),
        title: NODES_TITLE,
      },
      {
        path: 'bootstrap',
        loadChildren: () => import('@nodes/bootstrap/nodes-bootstrap.module').then(m => m.NodesBootstrapModule),
        title: NODES_TITLE,
      },
      {
        path: 'live',
        loadChildren: () => import('@nodes/live/nodes-live.module').then(m => m.NodesLiveModule),
        title: NODES_TITLE,
      },
      {
        path: '**',
        redirectTo: 'overview',
        pathMatch: 'full',
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
  exports: [RouterModule]
})
export class NodesRouting {}
