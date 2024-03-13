import { NgModule } from '@angular/core';

import { NetworkNodeDhtTableComponent } from './node-dht-table/network-node-dht-table.component';
import { SharedModule } from '@shared/shared.module';
import { NetworkNodeDhtComponent } from '@network/node-dht/network-node-dht.component';
import { HorizontalResizableContainerComponent, MinaJsonViewerComponent } from '@openmina/shared';
import {
  NetworkNodeDhtSidePanelComponent
} from '@network/node-dht/network-node-dht-side-panel/network-node-dht-side-panel.component';
import { NetworkNodeDhtRouting } from '@network/node-dht/network-node-dht.routing';
import { EffectsModule } from '@ngrx/effects';
import { NetworkNodeDhtEffects } from '@network/node-dht/network-node-dht.effects';
import { NetworkNodeDhtLineComponent } from './network-node-dht-line/network-node-dht-line.component';


@NgModule({
  declarations: [
    NetworkNodeDhtComponent,
    NetworkNodeDhtTableComponent,
    NetworkNodeDhtSidePanelComponent,
    NetworkNodeDhtLineComponent,
  ],
  imports: [
    SharedModule,
    NetworkNodeDhtRouting,
    HorizontalResizableContainerComponent,
    MinaJsonViewerComponent,
    EffectsModule.forFeature(NetworkNodeDhtEffects)
  ],
})
export class NetworkNodeDhtModule {}
