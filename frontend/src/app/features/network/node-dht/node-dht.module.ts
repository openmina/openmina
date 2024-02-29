import { NgModule } from '@angular/core';

import { NodeDhtRouting } from './node-dht.routing';
import { NodeDhtTableComponent } from './node-dht-table/node-dht-table.component';
import { NodeDhtSidePanelComponent } from './node-dht-side-panel/node-dht-side-panel.component';
import { SharedModule } from '@shared/shared.module';
import { NodeDhtComponent } from '@network/node-dht/node-dht.component';
import { HorizontalResizableContainerComponent, MinaJsonViewerComponent } from '@openmina/shared';


@NgModule({
  declarations: [
    NodeDhtComponent,
    NodeDhtTableComponent,
    NodeDhtSidePanelComponent,
  ],
  imports: [
    SharedModule,
    NodeDhtRouting,
    HorizontalResizableContainerComponent,
    MinaJsonViewerComponent,
  ],
})
export class NodeDhtModule {}
