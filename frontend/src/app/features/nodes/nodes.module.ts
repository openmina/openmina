import { NgModule } from '@angular/core';

import { NodesRouting } from './nodes.routing';
import { NodesComponent } from './nodes.component';
import { SharedModule } from '@shared/shared.module';


@NgModule({
  declarations: [
    NodesComponent
  ],
  imports: [
    SharedModule,
    NodesRouting
  ]
})
export class NodesModule {}
