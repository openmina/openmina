import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { BlockProductionWonSlotsComponent } from '@block-production/won-slots/block-production-won-slots.component';
import { BlockProductionWonSlotsRouting } from '@block-production/won-slots/block-production-won-slots.routing';
import {
  BlockProductionWonSlotsTableComponent,
} from '@block-production/won-slots/table/block-production-won-slots-table.component';
import {
  BlockProductionWonSlotsSidePanelComponent,
} from '@block-production/won-slots/side-panel/block-production-won-slots-side-panel.component';
import { EffectsModule } from '@ngrx/effects';
import { BlockProductionWonSlotsEffects } from '@block-production/won-slots/block-production-won-slots.effects';
import { HorizontalMenuComponent, HorizontalResizableContainerComponent } from '@openmina/shared';
import { LoadingSpinnerComponent } from '@shared/loading-spinner/loading-spinner.component';
import {
  BlockProductionWonSlotsFiltersComponent,
} from '@block-production/won-slots/filters/block-production-won-slots-filters.component';
import {
  BlockProductionWonSlotsCardsComponent,
} from '@block-production/won-slots/cards/block-production-won-slots-cards.component';
import { SharedModule } from '@shared/shared.module';


@NgModule({
  declarations: [
    BlockProductionWonSlotsComponent,
    BlockProductionWonSlotsTableComponent,
    BlockProductionWonSlotsSidePanelComponent,
    BlockProductionWonSlotsFiltersComponent,
    BlockProductionWonSlotsCardsComponent,
  ],
  imports: [
    CommonModule,
    BlockProductionWonSlotsRouting,
    EffectsModule.forFeature(BlockProductionWonSlotsEffects),
    HorizontalResizableContainerComponent,
    LoadingSpinnerComponent,
    HorizontalMenuComponent,
    SharedModule,
  ],
})
export class BlockProductionWonSlotsModule {}
