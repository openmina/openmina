import { NgModule } from '@angular/core';

import { SharedModule } from '@shared/shared.module';
import { NetworkNodeDhtComponent } from '@network/node-dht/network-node-dht.component';
import {
  HorizontalResizableContainerComponent,
  MinaJsonViewerComponent,
  MinaSidePanelStepperComponent
} from '@openmina/shared';
import {
  NetworkNodeDhtSidePanelComponent
} from '@network/node-dht/network-node-dht-side-panel/network-node-dht-side-panel.component';
import { NetworkNodeDhtRouting } from '@network/node-dht/network-node-dht.routing';
import { EffectsModule } from '@ngrx/effects';
import { NetworkNodeDhtEffects } from '@network/node-dht/network-node-dht.effects';
import { NetworkNodeDhtLineComponent } from './network-node-dht-line/network-node-dht-line.component';
import { NetworkNodeDhtTableComponent } from './network-node-dht-table/network-node-dht-table.component';
import { NetworkNodeDhtPeerDetailsComponent } from './network-node-dht-peer-details/network-node-dht-peer-details.component';
import { NetworkNodeDhtBootstrapStatsComponent } from './network-node-dht-bootstrap-stats/network-node-dht-bootstrap-stats.component';
import { NetworkNodeDhtBootstrapDetailsComponent } from './network-node-dht-bootstrap-details/network-node-dht-bootstrap-details.component';


@NgModule({
  declarations: [
    NetworkNodeDhtComponent,
    NetworkNodeDhtTableComponent,
    NetworkNodeDhtSidePanelComponent,
    NetworkNodeDhtLineComponent,
    NetworkNodeDhtPeerDetailsComponent,
    NetworkNodeDhtBootstrapStatsComponent,
    NetworkNodeDhtBootstrapDetailsComponent,
  ],
  imports: [
    SharedModule,
    NetworkNodeDhtRouting,
    HorizontalResizableContainerComponent,
    MinaJsonViewerComponent,
    EffectsModule.forFeature(NetworkNodeDhtEffects),
    MinaSidePanelStepperComponent
  ],
})
export class NetworkNodeDhtModule {}
