import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';

const routes: Routes = [
  {
    path: 'scenarios',
    loadChildren: () => import('./scenarios/testing-tool-scenarios.module').then(m => m.TestingToolScenariosModule),
  },
  {
    path: '**',
    redirectTo: 'scenarios',
    pathMatch: 'full',
  }
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class TestingToolRouting { }
