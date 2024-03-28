import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { DhtGraphComponent } from '@network/dht-graph/dht-graph.component';

const routes: Routes = [
  {
    path: '',
    component: DhtGraphComponent,
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class DhtGraphRouting {}
