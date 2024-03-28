import { NgModule } from '@angular/core';

import {
  NetworkDhtGraphBinaryTreeComponent,
} from './network-dht-graph-binary-tree/network-dht-graph-binary-tree.component';
import { DhtGraphComponent } from '@network/dht-graph/dht-graph.component';
import { DhtGraphRouting } from '@network/dht-graph/dht-graph.routing';
import { SharedModule } from '@shared/shared.module';


@NgModule({
  declarations: [
    DhtGraphComponent,
    NetworkDhtGraphBinaryTreeComponent,
  ],
  imports: [
    SharedModule,
    DhtGraphRouting,
  ],
})
export class DhtGraphModule {}
