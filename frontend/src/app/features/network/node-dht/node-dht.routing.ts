import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NodeDhtComponent } from '@network/node-dht/node-dht.component';

const routes: Routes = [
  {
    path: '',
    component: NodeDhtComponent,
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class NodeDhtRouting {}
