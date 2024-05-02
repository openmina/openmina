import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { BLOCK_PRODUCTION_TITLE } from '@app/app.routing';
import { BlockProductionWonSlotsComponent } from '@block-production/won-slots/block-production-won-slots.component';

const routes: Routes = [
  {
    path: '',
    component: BlockProductionWonSlotsComponent,
    title: BLOCK_PRODUCTION_TITLE,
    children: [
      {
        path: ':id',
        component: BlockProductionWonSlotsComponent,
        title: BLOCK_PRODUCTION_TITLE,
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
export class BlockProductionWonSlotsRouting {}
