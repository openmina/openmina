import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { SNARKS_TITLE } from '@app/app.routing';
import { SnarksComponent } from '@snarks/snarks.component';
import { CONFIG } from '@shared/constants/config';

const routes: Routes = [
  {
    path: '',
    component: SnarksComponent,
    children: [
      {
        path: 'scan-state',
        loadChildren: () => import('@snarks/scan-state/scan-state.module').then(m => m.ScanStateModule),
        title: SNARKS_TITLE,
      },
      {
        path: 'work-pool',
        loadChildren: () => import('@snarks/work-pool/snarks-work-pool.module').then(m => m.SnarksWorkPoolModule),
        title: SNARKS_TITLE,
      },
      {
        path: '**',
        redirectTo: CONFIG.globalConfig.features['snarks'].includes('work-pool') ? 'work-pool' : 'scan-state',
        pathMatch: 'full',
      },
    ],
  }
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class SnarksRouting {}
