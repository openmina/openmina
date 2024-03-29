import { NgModule } from '@angular/core';

import { NetworkBootstrapStatsRouting } from './network-bootstrap-stats.routing';
import { NetworkBootstrapStatsComponent } from './network-bootstrap-stats.component';
import { SharedModule } from '@shared/shared.module';
import {
  NetworkBootstrapStatsTableComponent,
} from '@network/bootstrap-stats/table/network-bootstrap-stats-table.component';
import {
  NetworkBootstrapStatsSidePanelComponent,
} from '@network/bootstrap-stats/side-panel/network-bootstrap-stats-side-panel.component';
import { EffectsModule } from '@ngrx/effects';
import { NetworkBootstrapStatsEffects } from '@network/bootstrap-stats/network-bootstrap-stats.effects';
import {
  CopyComponent,
  HorizontalMenuComponent,
  HorizontalResizableContainerComponent,
  MinaJsonViewerComponent,
} from '@openmina/shared';
import { LoadingSpinnerComponent } from '@shared/loading-spinner/loading-spinner.component';


@NgModule({
  declarations: [
    NetworkBootstrapStatsComponent,
    NetworkBootstrapStatsSidePanelComponent,
    NetworkBootstrapStatsTableComponent,
  ],
  imports: [
    SharedModule,
    NetworkBootstrapStatsRouting,
    EffectsModule.forFeature(NetworkBootstrapStatsEffects),
    HorizontalResizableContainerComponent,
    CopyComponent,
    HorizontalMenuComponent,
    MinaJsonViewerComponent,
    LoadingSpinnerComponent,
  ],
})
export class NetworkBootstrapStatsModule {}
