import { NgModule } from '@angular/core';

import { SharedModule } from '@shared/shared.module';
import { NetworkNodeDhtComponent } from '@network/node-dht/network-node-dht.component';
import {
  CopyComponent,
  HorizontalResizableContainerComponent,
  MinaJsonViewerComponent,
  MinaSidePanelStepperComponent,
} from '@openmina/shared';
import {
  NetworkNodeDhtSidePanelComponent,
} from '@network/node-dht/network-node-dht-side-panel/network-node-dht-side-panel.component';
import { NetworkNodeDhtRouting } from '@network/node-dht/network-node-dht.routing';
import { EffectsModule } from '@ngrx/effects';
import { NetworkNodeDhtEffects } from '@network/node-dht/network-node-dht.effects';
import { NetworkNodeDhtLineComponent } from './network-node-dht-line/network-node-dht-line.component';
import { NetworkNodeDhtTableComponent } from './network-node-dht-table/network-node-dht-table.component';


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
    EffectsModule.forFeature(NetworkNodeDhtEffects),
    MinaSidePanelStepperComponent,
    CopyComponent,
  ],
})
export class NetworkNodeDhtModule {}
