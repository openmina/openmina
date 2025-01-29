import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { LeaderboardRouting } from './leaderboard.routing';
import { LeaderboardFiltersComponent } from '@leaderboard/leaderboard-filters/leaderboard-filters.component';
import { LeaderboardHeaderComponent } from '@leaderboard/leaderboard-header/leaderboard-header.component';
import { LeaderboardPageComponent } from '@leaderboard/leaderboard-page/leaderboard-page.component';
import { LeaderboardTableComponent } from '@leaderboard/leaderboard-table/leaderboard-table.component';
import { LeaderboardTitleComponent } from '@leaderboard/leaderboard-title/leaderboard-title.component';
import { CopyComponent, OpenminaSharedModule } from '@openmina/shared';
import { LoadingSpinnerComponent } from '@shared/loading-spinner/loading-spinner.component';
import { EffectsModule } from '@ngrx/effects';
import { LeaderboardEffects } from '@leaderboard/leaderboard.effects';
import { LeaderboardFooterComponent } from '@leaderboard/leaderboard-footer/leaderboard-footer.component';
import { LeaderboardLandingPageComponent } from '@leaderboard/leaderboard-landing-page/leaderboard-landing-page.component';


@NgModule({
  declarations: [
    LeaderboardPageComponent,
    LeaderboardFiltersComponent,
    LeaderboardHeaderComponent,
    LeaderboardTableComponent,
    LeaderboardTitleComponent,
    LeaderboardFooterComponent,
    LeaderboardLandingPageComponent,
  ],
  imports: [
    CommonModule,
    LeaderboardRouting,
    CopyComponent,
    OpenminaSharedModule,
    LoadingSpinnerComponent,
    EffectsModule.forFeature(LeaderboardEffects),
  ],
  exports: [
    LeaderboardLandingPageComponent,
  ],
})
export class LeaderboardModule {}
