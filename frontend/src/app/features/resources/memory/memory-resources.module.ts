import { NgModule } from '@angular/core';

import { MemoryResourcesRouting } from '@resources/memory/memory-resources.routing';
import { MemoryResourcesComponent } from '@resources/memory/memory-resources.component';
import { SharedModule } from '@shared/shared.module';
import { EffectsModule } from '@ngrx/effects';
import { MemoryResourcesEffects } from '@resources/memory/memory-resources.effects';
import {
  MemoryResourcesTableComponent,
} from '@resources/memory/memory-resources-table/memory-resources-table.component';
import { HorizontalMenuComponent } from '@openmina/shared';
import { ResourcesSizePipe } from '@resources/memory/memory-resources.pipe';
import {
  MemoryResourcesTreemapComponent,
} from '@resources/memory/memory-resources-treemap/memory-resources-treemap.component';
import { MemoryResourcesToolbarComponent } from './memory-resources-toolbar/memory-resources-toolbar.component';
import { MemoryResourcesService } from '@resources/memory/memory-resources.service';


@NgModule({
  declarations: [
    MemoryResourcesComponent,
    MemoryResourcesTableComponent,
    MemoryResourcesTreemapComponent,
    ResourcesSizePipe,
    MemoryResourcesToolbarComponent,
  ],
  imports: [
    MemoryResourcesRouting,
    SharedModule,
    EffectsModule.forFeature(MemoryResourcesEffects),
    HorizontalMenuComponent,
  ],
  providers: [
    ResourcesSizePipe,
    MemoryResourcesService,
    MemoryResourcesEffects,
  ],
})
export class MemoryResourcesModule {}
