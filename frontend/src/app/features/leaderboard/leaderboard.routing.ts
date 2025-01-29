import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { LeaderboardPageComponent } from '@leaderboard/leaderboard-page/leaderboard-page.component';

const routes: Routes = [
  {
    path: 'leaderboard',
    component: LeaderboardPageComponent,
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
