import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';

const routes: Routes = [
  {
    path: 'memory',
    loadChildren: () => import('@resources/memory/memory-resources.module').then(m => m.MemoryResourcesModule),
  },
  {
    path: '**',
    redirectTo: 'memory',
    pathMatch: 'full',
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class ResourcesRouting {}
