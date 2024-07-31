import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { BenchmarksWalletsComponent } from '@benchmarks/wallets/benchmarks-wallets.component';
import { BENCHMARKS_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    path: '',
    component: BenchmarksWalletsComponent,
    title: BENCHMARKS_TITLE,
  },
  {
    path: '**',
    pathMatch: 'full',
    redirectTo: '',
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class BenchmarksWalletsRouting {}
