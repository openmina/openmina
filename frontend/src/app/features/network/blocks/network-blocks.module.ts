import { NgModule } from '@angular/core';

import { NetworkBlocksRouting } from './network-blocks.routing';
import { SharedModule } from '@shared/shared.module';
import { NetworkBlocksComponent } from '@network/blocks/network-blocks.component';
import { NetworkBlocksTableComponent } from './network-blocks-table/network-blocks-table.component';
import { NetworkBlocksGraphComponent } from './network-blocks-graph/network-blocks-graph.component';
import { EffectsModule } from '@ngrx/effects';
import { NetworkBlocksEffects } from '@network/blocks/network-blocks.effects';
import { NetworkBlocksSidePanelComponent } from './network-blocks-side-panel/network-blocks-side-panel.component';
import { NetworkBlocksToolbarComponent } from './network-blocks-toolbar/network-blocks-toolbar.component';
import { CopyComponent, HorizontalResizableContainerComponent } from '@openmina/shared';


@NgModule({
  declarations: [
    NetworkBlocksComponent,
    NetworkBlocksTableComponent,
    NetworkBlocksGraphComponent,
    NetworkBlocksSidePanelComponent,
    NetworkBlocksToolbarComponent,
  ],
  imports: [
    SharedModule,
    CopyComponent,
    NetworkBlocksRouting,
    EffectsModule.forFeature([NetworkBlocksEffects]),
    HorizontalResizableContainerComponent,
  ],
  exports: [
    NetworkBlocksGraphComponent,
  ],
})
export class NetworkBlocksModule {}
