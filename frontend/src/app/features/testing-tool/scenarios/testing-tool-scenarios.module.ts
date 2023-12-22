import { NgModule } from '@angular/core';

import { TestingToolScenariosRouting } from './testing-tool-scenarios.routing';
import { TestingToolScenariosComponent } from './testing-tool-scenarios.component';
import { ScenariosStepsTableComponent } from './scenarios-steps-table/scenarios-steps-table.component';
import {
  ScenariosEventTracesTableComponent
} from './scenarios-event-traces-table/scenarios-event-traces-table.component';
import { ScenariosAddStepComponent } from './scenarios-add-step/scenarios-add-step.component';
import { ScenariosStepsToolbarComponent } from './scenarios-steps-toolbar/scenarios-steps-toolbar.component';
import { ScenariosStepsFooterComponent } from './scenarios-steps-footer/scenarios-steps-footer.component';
import { SharedModule } from '@shared/shared.module';
import { CopyComponent, HorizontalResizableContainerComponent } from '@openmina/shared';
import { EffectsModule } from '@ngrx/effects';
import { TestingToolScenariosEffects } from '@testing-tool/scenarios/testing-tool-scenarios.effects';
import { FormsModule } from '@angular/forms';


@NgModule({
  declarations: [
    TestingToolScenariosComponent,
    ScenariosStepsTableComponent,
    ScenariosEventTracesTableComponent,
    ScenariosAddStepComponent,
    ScenariosStepsToolbarComponent,
    ScenariosStepsFooterComponent
  ],
	imports: [
		TestingToolScenariosRouting,
		SharedModule,
		HorizontalResizableContainerComponent,
		CopyComponent,
		EffectsModule.forFeature(TestingToolScenariosEffects),
		FormsModule,
	],
})
export class TestingToolScenariosModule {}
