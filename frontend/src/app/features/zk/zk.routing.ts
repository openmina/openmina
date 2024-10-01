import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { STATE_TITLE } from '@app/app.routing';
import { StateComponent } from '@app/features/state/state.component';
import { ZkComponent } from '@app/features/zk/zk.component';

const routes: Routes = [
  {
    path: '',
    component: ZkComponent,
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
export class ZKRouting {}
