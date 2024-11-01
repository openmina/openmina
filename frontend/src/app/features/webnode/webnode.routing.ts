import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { WebnodeComponent } from '@app/features/webnode/webnode.component';

const routes: Routes = [
  {
    path: '',
    component: WebnodeComponent,
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class WebnodeRouting {}
