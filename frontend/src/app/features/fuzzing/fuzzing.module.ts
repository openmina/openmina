import { NgModule } from '@angular/core';

import { FuzzingComponent } from '@fuzzing/fuzzing.component';
import { FuzzingFilesTableComponent } from '@fuzzing/fuzzing-files-table/fuzzing-files-table.component';
import { FuzzingCodeComponent } from '@fuzzing/fuzzing-code/fuzzing-code.component';
import { FuzzingRouting } from '@fuzzing/fuzzing.routing';
import { SharedModule } from '@shared/shared.module';
import { EffectsModule } from '@ngrx/effects';
import { FuzzingEffects } from '@fuzzing/fuzzing.effects';
import { FuzzingToolbarComponent } from '@fuzzing/fuzzing-toolbar/fuzzing-toolbar.component';
import { FuzzingDirectoriesTableComponent } from '@fuzzing/fuzzing-directories-table/fuzzing-directories-table.component';
import { CopyComponent, HorizontalResizableContainerComponent } from '@openmina/shared';
import { LoadingSpinnerComponent } from '@shared/loading-spinner/loading-spinner.component';


@NgModule({
  declarations: [
    FuzzingComponent,
    FuzzingFilesTableComponent,
    FuzzingCodeComponent,
    FuzzingToolbarComponent,
    FuzzingDirectoriesTableComponent,
  ],
  imports: [
    SharedModule,
    FuzzingRouting,
    EffectsModule.forFeature(FuzzingEffects),
    HorizontalResizableContainerComponent,
    CopyComponent,
    LoadingSpinnerComponent,
  ],
})
export class FuzzingModule {}
