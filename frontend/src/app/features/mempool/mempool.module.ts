import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { MempoolRouting } from './mempool.routing';
import { MempoolComponent } from './mempool.component';


@NgModule({
  declarations: [
    MempoolComponent,
  ],
  imports: [
    CommonModule,
    MempoolRouting,
  ],
})
export class MempoolModule {}
