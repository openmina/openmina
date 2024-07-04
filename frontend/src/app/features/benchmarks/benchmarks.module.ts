import { NgModule } from '@angular/core';
import { BenchmarksComponent } from './benchmarks.component';
import { BenchmarksRouting } from '@benchmarks/benchmarks.routing';


@NgModule({
  declarations: [
    BenchmarksComponent,
  ],
  imports: [
    BenchmarksRouting,
  ],
})
export class BenchmarksModule {}
