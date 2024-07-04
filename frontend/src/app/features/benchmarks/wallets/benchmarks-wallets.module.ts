import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { BenchmarksWalletsComponent } from './benchmarks-wallets.component';
import { SharedModule } from '@shared/shared.module';
import { EffectsModule } from '@ngrx/effects';
import { BenchmarksWalletsEffects } from '@benchmarks/wallets/benchmarks-wallets.effects';
import {
  BenchmarksWalletsTableComponent,
} from '@benchmarks/wallets/benchmarks-wallets-table/benchmarks-wallets-table.component';
import {
  BenchmarksWalletsToolbarComponent,
} from '@benchmarks/wallets/benchmarks-wallets-toolbar/benchmarks-wallets-toolbar.component';
import { BenchmarksWalletsRouting } from '@benchmarks/wallets/benchmarks-wallets.routing';
import { CopyComponent, HorizontalMenuComponent } from '@openmina/shared';


@NgModule({
  declarations: [
    BenchmarksWalletsComponent,
    BenchmarksWalletsTableComponent,
    BenchmarksWalletsToolbarComponent,
  ],
  imports: [
    CommonModule,
    SharedModule,
    CopyComponent,
    BenchmarksWalletsRouting,
    EffectsModule.forFeature([BenchmarksWalletsEffects]),
    HorizontalMenuComponent,
  ],
})
export class BenchmarksWalletsModule {}
