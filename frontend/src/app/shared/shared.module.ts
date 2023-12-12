import { NgModule } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { OpenminaEagerSharedModule, OpenminaSharedModule } from '@openmina/shared';
import { CommonModule } from '@angular/common';


const MODULES = [
  CommonModule,
  OpenminaEagerSharedModule,
  OpenminaSharedModule,
  ReactiveFormsModule,
];

@NgModule({
  imports: [
    ...MODULES,
  ],
  exports: [
    ...MODULES,
  ],
})
export class SharedModule {}
