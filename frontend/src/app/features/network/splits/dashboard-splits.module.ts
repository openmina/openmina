import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { DashboardSplitsRouting } from './dashboard-splits.routing';
import { DashboardSplitsGraphComponent } from './dashboard-splits-graph/dashboard-splits-graph.component';
import { DashboardSplitsToolbarComponent } from './dashboard-splits-toolbar/dashboard-splits-toolbar.component';
import { DashboardSplitsSidePanelComponent } from './dashboard-splits-side-panel/dashboard-splits-side-panel.component';
import { EffectsModule } from '@ngrx/effects';
import { DashboardSplitsSidePanelTableComponent } from './dashboard-splits-side-panel-table/dashboard-splits-side-panel-table.component';
import { CopyComponent, HorizontalMenuComponent, HorizontalResizableContainerComponent } from '@openmina/shared';
import { DashboardSplitsComponent } from '@network/splits/dashboard-splits.component';
import { SharedModule } from '@shared/shared.module';
import { DashboardSplitsEffects } from '@network/splits/dashboard-splits.effects';


@NgModule({
  declarations: [
    DashboardSplitsComponent,
    DashboardSplitsGraphComponent,
    DashboardSplitsToolbarComponent,
    DashboardSplitsSidePanelComponent,
    DashboardSplitsSidePanelTableComponent,
  ],
  imports: [
    CommonModule,
    CopyComponent,
    DashboardSplitsRouting,
    SharedModule,
    EffectsModule.forFeature([DashboardSplitsEffects]),
    HorizontalMenuComponent,
    HorizontalResizableContainerComponent,
  ],
})
export class DashboardSplitsModule {}
