import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { BlockProductionComponent } from './block-production.component';
import { BLOCK_PRODUCTION_TITLE } from '@app/app.routing';
import { blockProductionGuard } from '@block-production/block-production.guard';

const routes: Routes = [
  {
    path: '',
    component: BlockProductionComponent,
    title: BLOCK_PRODUCTION_TITLE,
    children: [
      {
        path: 'overview',
        canActivate: [blockProductionGuard],
        loadChildren: () => import('./overview/block-production-overview.module').then(m => m.BlockProductionOverviewModule),
      },
      {
        path: 'won-slots',
        canActivate: [blockProductionGuard],
        loadChildren: () => import('./won-slots/block-production-won-slots.module').then(m => m.BlockProductionWonSlotsModule),
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
