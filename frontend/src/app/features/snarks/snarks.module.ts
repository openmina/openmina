import { NgModule } from '@angular/core';

import { SnarksRouting } from './snarks.routing';
import { SnarksComponent } from './snarks.component';
import { SharedModule } from '@shared/shared.module';


@NgModule({
  declarations: [
    SnarksComponent
  ],
  imports: [
    SharedModule,
    SnarksRouting
  ]
})
export class SnarksModule { }
