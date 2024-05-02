import { NgModule } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { OpenminaEagerSharedModule, OpenminaSharedModule } from '@openmina/shared';
import { CommonModule } from '@angular/common';
import { MinaCardComponent } from '@shared/components/mina-card/mina-card.component';


const MODULES = [
  CommonModule,
  OpenminaEagerSharedModule,
  OpenminaSharedModule,
  ReactiveFormsModule,
];

const COMPONENTS = [
  MinaCardComponent,
];

@NgModule({
  imports: [
    ...MODULES,
  ],
  declarations: [
    ...COMPONENTS,
  ],
  exports: [
    ...MODULES,
    ...COMPONENTS,
  ],
})
export class SharedModule {}
