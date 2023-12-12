import { NgModule } from '@angular/core';

import { ScanStateRouting } from './scan-state.routing';
import { ScanStateComponent } from './scan-state.component';
import { ScanStateTreeChartComponent } from './scan-state-tree-chart/scan-state-tree-chart.component';
import { ScanStateTreeListComponent } from './scan-state-tree-list/scan-state-tree-list.component';
import { SharedModule } from '@shared/shared.module';
import { EffectsModule } from '@ngrx/effects';
import { ScanStateEffects } from '@snarks/scan-state/scan-state.effects';
import { ScanStateToolbarComponent } from './scan-state-toolbar/scan-state-toolbar.component';
import {
  CopyComponent,
  HorizontalMenuComponent,
  HorizontalResizableContainerComponent, JsonConsoleComponent,
  MinaJsonViewerComponent,
  MinaSidePanelStepperComponent
} from '@openmina/shared';
import { ScanStateSidePanelComponent } from './scan-state-side-panel/scan-state-side-panel.component';
import { ScanStateDetailsComponent } from './scan-state-details/scan-state-details.component';
import { ScanStateJobDetailsComponent } from './scan-state-job-details/scan-state-job-details.component';


@NgModule({
  declarations: [
    ScanStateComponent,
    ScanStateTreeChartComponent,
    ScanStateTreeListComponent,
    ScanStateToolbarComponent,
    ScanStateSidePanelComponent,
    ScanStateDetailsComponent,
    ScanStateJobDetailsComponent,
  ],
  imports: [
    SharedModule,
    ScanStateRouting,
    EffectsModule.forFeature(ScanStateEffects),
    CopyComponent,
    HorizontalMenuComponent,
    HorizontalResizableContainerComponent,
    MinaSidePanelStepperComponent,
    MinaJsonViewerComponent,
    JsonConsoleComponent
  ]
})
export class ScanStateModule {}
