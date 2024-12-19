import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { WebNodeComponent } from './web-node.component';

const routes: Routes = [
  {
    path: '',
    component: WebNodeComponent,
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class WebNodeRouting {}
