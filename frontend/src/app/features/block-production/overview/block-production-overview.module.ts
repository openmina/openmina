import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { BlockProductionOverviewRouting } from './block-production-overview.routing';
import { BlockProductionOverviewComponent } from './block-production-overview.component';
import { BlockProductionSlotsComponent } from './block-production-slots/block-production-slots.component';
import { HorizontalResizableContainerComponent } from '@openmina/shared';


@NgModule({
  declarations: [
    BlockProductionOverviewComponent,
    BlockProductionSlotsComponent,
  ],
  imports: [
    CommonModule,
    BlockProductionOverviewRouting,
    HorizontalResizableContainerComponent,
  ],
})
export class BlockProductionOverviewModule {}
