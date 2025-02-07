import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect } from '@openmina/shared';
import { map, switchMap } from 'rxjs';
import { catchErrorAndRepeat2 } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Store } from '@ngrx/store';
import { BaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import { LeaderboardActions } from '@leaderboard/leaderboard.actions';
import { LeaderboardService } from '@leaderboard/leaderboard.service';

@Injectable({
  providedIn: 'root',
})
export class LeaderboardEffects extends BaseEffect {

  readonly getHeartbeats$: Effect;

  constructor(private actions$: Actions,
              private leaderboardService: LeaderboardService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getHeartbeats$ = createEffect(() => this.actions$.pipe(
      ofType(LeaderboardActions.getHeartbeats),
      this.latestActionState(),
      switchMap(() => this.leaderboardService.getHeartbeatsSummaries()),
      map(heartbeatSummaries => LeaderboardActions.getHeartbeatsSuccess({ heartbeatSummaries })),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, LeaderboardActions.getHeartbeatsSuccess({ heartbeatSummaries: [] })),
    ));
  }
}
