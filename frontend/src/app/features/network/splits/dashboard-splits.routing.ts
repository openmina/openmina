import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { DashboardSplitsComponent } from '@network/splits/dashboard-splits.component';
import { NETWORK_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    path: '',
    component: DashboardSplitsComponent,
    children: [
      {
        path: ':addr',
        component: DashboardSplitsComponent,
        title: NETWORK_TITLE,
      },
    ],
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class DashboardSplitsRouting {}
