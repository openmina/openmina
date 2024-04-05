import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { BlockProductionRouting } from './block-production.routing';
import { BlockProductionComponent } from './block-production.component';


@NgModule({
  declarations: [
    BlockProductionComponent,
  ],
  imports: [
    CommonModule,
    BlockProductionRouting,
  ],
})
export class BlockProductionModule {}
