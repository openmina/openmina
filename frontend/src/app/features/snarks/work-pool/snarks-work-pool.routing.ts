import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { SNARKS_TITLE } from '@app/app.routing';
import { SnarksWorkPoolComponent } from '@snarks/work-pool/snarks-work-pool.component';


const routes: Routes = [
  {
    path: '',
    component: SnarksWorkPoolComponent,
    children: [
      {
        path: ':id',
        component: SnarksWorkPoolComponent,
        title: SNARKS_TITLE,
      },
    ],
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class SnarksWorkPoolRouting {}
