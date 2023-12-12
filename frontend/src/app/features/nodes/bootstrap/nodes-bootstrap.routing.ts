import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NODES_TITLE } from '@app/app.routing';
import { NodesBootstrapComponent } from '@nodes/bootstrap/nodes-bootstrap.component';

const routes: Routes = [
  {
    path: '',
    component: NodesBootstrapComponent,
    children: [
      {
        path: ':index',
        component: NodesBootstrapComponent,
        title: NODES_TITLE,
      },
    ],
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class NodesBootstrapRouting {}
