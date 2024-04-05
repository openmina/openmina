import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { BlockProductionComponent } from './block-production.component';
import { BLOCK_PRODUCTION_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    path: '',
    component: BlockProductionComponent,
    title: BLOCK_PRODUCTION_TITLE,
    children: [
      {
        path: 'overview',
        loadChildren: () => import('./overview/block-production-overview.module').then(m => m.BlockProductionOverviewModule),
      },
      {
        path: '',
        pathMatch: 'full',
        redirectTo: 'overview',
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
export class BlockProductionRouting {}
