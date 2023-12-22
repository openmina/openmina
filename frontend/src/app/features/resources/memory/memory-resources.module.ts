import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { MemoryResourcesRouting } from '@resources/memory/memory-resources.routing';
import { MemoryResourcesTreeMapComponent } from '@resources/memory/memory-resources-tree-map/memory-resources-tree-map.component';
import { MemoryResourcesComponent } from '@resources/memory/memory-resources.component';


@NgModule({
  declarations: [
    MemoryResourcesComponent,
    MemoryResourcesTreeMapComponent,
  ],
  imports: [
    CommonModule,
    MemoryResourcesRouting,
  ],
})
export class MemoryResourcesModule {}
