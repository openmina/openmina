import { NgModule } from '@angular/core';

import { StateRouting } from './state.routing';
import { StateComponent } from './state.component';
import { SharedModule } from '@shared/shared.module';


@NgModule({
  declarations: [
    StateComponent
  ],
  imports: [
    SharedModule,
    StateRouting
  ]
})
export class StateModule { }
