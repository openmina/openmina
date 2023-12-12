import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { MemoryResourcesComponent } from '@resources/memory/memory-resources.component';

const routes: Routes = [
  {
    path: '',
    component: MemoryResourcesComponent,
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
export class MemoryResourcesRouting {}
