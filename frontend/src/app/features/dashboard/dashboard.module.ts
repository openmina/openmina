import { NgModule } from '@angular/core';

import { DashboardRouting } from '@dashboard/dashboard.routing';
import { DashboardComponent } from '@dashboard/dashboard.component';
import { SharedModule } from '@shared/shared.module';
import { EffectsModule } from '@ngrx/effects';
import { DashboardEffects } from '@dashboard/dashboard.effects';
import { LoadingSpinnerComponent } from '@shared/loading-spinner/loading-spinner.component';
import { CopyComponent } from '@openmina/shared';
import { DashboardNetworkComponent } from './dashboard-network/dashboard-network.component';
import { DashboardLedgerComponent } from './dashboard-ledger/dashboard-ledger.component';
import { DashboardBlocksSyncComponent } from './dashboard-blocks-sync/dashboard-blocks-sync.component';
import { DashboardPeersMinimalTableComponent } from './dashboard-peers-minimal-table/dashboard-peers-minimal-table.component';
import { BlockProductionPillComponent } from '@app/layout/block-production-pill/block-production-pill.component';


@NgModule({
  declarations: [
    DashboardComponent,
    DashboardNetworkComponent,
    DashboardLedgerComponent,
    DashboardBlocksSyncComponent,
    DashboardPeersMinimalTableComponent,
  ],
  imports: [
    SharedModule,
    DashboardRouting,
    EffectsModule.forFeature(DashboardEffects),
    LoadingSpinnerComponent,
    CopyComponent,
    BlockProductionPillComponent,
  ],
})
export class DashboardModule {}
