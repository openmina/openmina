import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ResourcesComponent } from '@resources/resources.component';
import { ResourcesRouting } from '@resources/resources.routing';



@NgModule({
  declarations: [
    ResourcesComponent,
  ],
  imports: [
    CommonModule,
    ResourcesRouting
  ]
})
export class ResourcesModule { }
