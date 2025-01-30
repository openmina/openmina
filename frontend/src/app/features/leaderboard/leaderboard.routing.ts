import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { LeaderboardPageComponent } from '@leaderboard/leaderboard-page/leaderboard-page.component';
import { LeaderboardDetailsComponent } from '@leaderboard/leaderboard-details/leaderboard-details.component';
import { LeaderboardPrivacyPolicyComponent } from '@leaderboard/leaderboard-privacy-policy/leaderboard-privacy-policy.component';
import { LeaderboardTermsAndConditionsComponent } from '@leaderboard/leaderboard-terms-and-conditions/leaderboard-terms-and-conditions.component';
import { LeaderboardImpressumComponent } from '@leaderboard/leaderboard-impressum/leaderboard-impressum.component';

const routes: Routes = [
  {
    path: 'leaderboard',
    component: LeaderboardPageComponent,
  },
  {
    path: 'leaderboard/details',
    component: LeaderboardDetailsComponent,
  },
  {
    path: 'leaderboard/impressum',
    component: LeaderboardImpressumComponent,
  },
  {
    path: 'leaderboard/privacy-policy',
    component: LeaderboardPrivacyPolicyComponent,
  },
  {
    path: 'leaderboard/terms-and-conditions',
    component: LeaderboardTermsAndConditionsComponent,
  },
  {
    path: '**',
    redirectTo: '',
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class LeaderboardRouting {}
