import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import {
  BlockProductionOverviewComponent,
} from '@app/features/block-production/overview/block-production-overview.component';
import { BLOCK_PRODUCTION_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    path: '',
    component: BlockProductionOverviewComponent,
    title: BLOCK_PRODUCTION_TITLE,
    children: [
      {
        path: ':epoch',
        component: BlockProductionOverviewComponent,
        title: BLOCK_PRODUCTION_TITLE,
        children: [
          {
            path: ':slot',
            component: BlockProductionOverviewComponent,
            title: BLOCK_PRODUCTION_TITLE,
          },
        ],
      },
    ],
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
export class BlockProductionOverviewRouting {}
