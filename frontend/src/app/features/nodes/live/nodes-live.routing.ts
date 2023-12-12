import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NODES_TITLE } from '@app/app.routing';
import { NodesLiveComponent } from '@nodes/live/nodes-live.component';

const routes: Routes = [
  {
    path: '',
    component: NodesLiveComponent,
    children: [
      {
        path: ':bestTip',
        component: NodesLiveComponent,
        title: NODES_TITLE,
      },
    ],
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class NodesLiveRouting {}
