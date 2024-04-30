import { NgModule } from '@angular/core';

import { BlockProductionOverviewRouting } from './block-production-overview.routing';
import { BlockProductionOverviewComponent } from './block-production-overview.component';
import {
  CopyComponent,
  HorizontalMenuComponent,
  HorizontalResizableContainerComponent,
  MinaJsonViewerComponent,
} from '@openmina/shared';
import { SharedModule } from '@shared/shared.module';
import {
  BlockProductionOverviewSlotsComponent,
} from '@block-production/overview/slots/block-production-overview-slots.component';
import {
  BlockProductionOverviewToolbarComponent,
} from '@block-production/overview/toolbar/block-production-overview-toolbar.component';
import {
  BlockProductionOverviewSidePanelComponent,
} from '@block-production/overview/side-panel/block-production-overview-side-panel.component';
import { PaginationComponent } from '@shared/pagination/pagination.component';
import { EffectsModule } from '@ngrx/effects';
import { BlockProductionOverviewEffects } from '@block-production/overview/block-production-overview.effects';
import {
  BlockProductionOverviewEpochGraphsComponent,
} from '@block-production/overview/epoch-graphs/block-production-overview-epoch-graphs.component';
import { LoadingSpinnerComponent } from '@shared/loading-spinner/loading-spinner.component';

@NgModule({
  declarations: [
    BlockProductionOverviewComponent,
    BlockProductionOverviewSlotsComponent,
    BlockProductionOverviewToolbarComponent,
    BlockProductionOverviewSidePanelComponent,
    BlockProductionOverviewEpochGraphsComponent,
  ],
  imports: [
    SharedModule,
    BlockProductionOverviewRouting,
    HorizontalResizableContainerComponent,
    PaginationComponent,
    HorizontalMenuComponent,
    EffectsModule.forFeature(BlockProductionOverviewEffects),
    CopyComponent,
    MinaJsonViewerComponent,
    LoadingSpinnerComponent,
  ],
})
export class BlockProductionOverviewModule {}
