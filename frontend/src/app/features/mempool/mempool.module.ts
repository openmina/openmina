import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { MempoolRouting } from './mempool.routing';
import { MempoolComponent } from './mempool.component';
import {
  HorizontalMenuComponent,
  HorizontalResizableContainerComponent,
  MinaJsonViewerComponent,
} from '@openmina/shared';
import { MempoolTableComponent } from '@app/features/mempool/table/mempool-table.component';
import { MempoolFiltersComponent } from '@app/features/mempool/filters/mempool-filters.component';
import { MempoolWarningsComponent } from '@app/features/mempool/warnings/mempool-warnings.component';
import {
  MempoolAddTransactionComponent,
} from '@app/features/mempool/add-transaction/mempool-add-transaction.component';
import { MempoolSidePanelComponent } from '@app/features/mempool/side-panel/mempool-side-panel.component';
import { EffectsModule } from '@ngrx/effects';
import { MempoolEffects } from '@app/features/mempool/mempool.effects';
import { SharedModule } from '@shared/shared.module';


@NgModule({
  declarations: [
    MempoolComponent,
    MempoolTableComponent,
    MempoolFiltersComponent,
    MempoolWarningsComponent,
    MempoolAddTransactionComponent,
    MempoolSidePanelComponent,
  ],
  imports: [
    CommonModule,
    MempoolRouting,
    SharedModule,
    HorizontalResizableContainerComponent,
    EffectsModule.forFeature(MempoolEffects),
    MinaJsonViewerComponent,
    HorizontalMenuComponent,
  ],
})
export class MempoolModule {}
