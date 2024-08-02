import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { MempoolComponent } from '@app/features/mempool/mempool.component';

const routes: Routes = [
  {
    path: '',
    component: MempoolComponent,
    children: [
      {
        path: ':id',
        component: MempoolComponent,
      },
    ],
  },
  {
    path: '**',
    redirectTo: '',
    pathMatch: 'full',
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class MempoolRouting {}
