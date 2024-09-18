import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ZkComponent } from './zk.component';
import { RouterModule } from '@angular/router';
import { ZKRouting } from '@app/features/zk/zk.routing';


@NgModule({
  declarations: [
    ZkComponent,
  ],
  imports: [
    CommonModule,
    ZKRouting,
  ],
})
export class ZkModule {}
