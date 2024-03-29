import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NetworkNodeDhtComponent } from '@network/node-dht/network-node-dht.component';

const routes: Routes = [
  {
    path: '',
    component: NetworkNodeDhtComponent,
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class NetworkNodeDhtRouting {
}
