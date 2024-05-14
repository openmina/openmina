import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect } from '@openmina/shared';
import { EMPTY, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat2 } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Store } from '@ngrx/store';
import { Router } from '@angular/router';
import { BaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import { BlockProductionOverviewActions } from '@block-production/overview/block-production-overview.actions';
import { BlockProductionOverviewService } from '@block-production/overview/block-production-overview.service';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import {
  BlockProductionOverviewEpochDetails,
} from '@shared/types/block-production/overview/block-production-overview-epoch-details.type';
import { Routes } from '@shared/enums/routes.enum';
import { BlockProductionSlot } from '@shared/types/block-production/overview/block-production-overview-slot.type';
import { BlockProductionModule } from '@block-production/block-production.module';

@Injectable({
  providedIn: BlockProductionModule,
})
export class BlockProductionOverviewEffects extends BaseEffect {

  readonly getActiveEpochDetails$: Effect;
  readonly getEpochs$: Effect;
  readonly getActiveEpochSlots$: Effect;
  readonly getRewardsStats$: Effect;

  constructor(private router: Router,
              private actions$: Actions,
              private bpOverviewService: BlockProductionOverviewService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getActiveEpochDetails$ = createEffect(() => this.actions$.pipe(
      ofType(BlockProductionOverviewActions.getEpochDetails, BlockProductionOverviewActions.close),
      this.latestActionState(),
      switchMap(({ action }) => action.type === BlockProductionOverviewActions.close.type ? EMPTY : this.bpOverviewService.getEpochDetails(action.epochNumber)),
      tap(response => this.router.navigate([Routes.BLOCK_PRODUCTION, Routes.OVERVIEW, response.epochNumber], { queryParamsHandling: 'merge' })),
      switchMap((payload: BlockProductionOverviewEpochDetails) => [
        BlockProductionOverviewActions.getEpochDetailsSuccess({ details: payload }),
        BlockProductionOverviewActions.getEpochs({ epochNumber: payload.epochNumber }),
        BlockProductionOverviewActions.getSlots({ epochNumber: payload.epochNumber }),
      ]),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, BlockProductionOverviewActions.getEpochDetailsSuccess(undefined)),
    ));

    this.getEpochs$ = createEffect(() => this.actions$.pipe(
      ofType(BlockProductionOverviewActions.getEpochs, BlockProductionOverviewActions.close),
      this.latestActionState(),
      switchMap(({ action }) =>
        action.type === BlockProductionOverviewActions.close.type
          ? EMPTY
          : this.bpOverviewService.getEpochs(action.epochNumber),
      ),
      map((epochs: BlockProductionOverviewEpoch[]) => BlockProductionOverviewActions.getEpochsSuccess({ epochs })),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, BlockProductionOverviewActions.getEpochsSuccess({ epochs: [] })),
    ));

    this.getActiveEpochSlots$ = createEffect(() => this.actions$.pipe(
      ofType(BlockProductionOverviewActions.getSlots, BlockProductionOverviewActions.close),
      this.latestActionState(),
      switchMap(({ state, action }) =>
        action.type === BlockProductionOverviewActions.close.type
          ? EMPTY
          : this.bpOverviewService.getSlots(state.blockProduction.overview.activeEpochNumber).pipe(
            map((slots: BlockProductionSlot[]) => BlockProductionOverviewActions.getSlotsSuccess({ slots })),
          ),
      ),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, BlockProductionOverviewActions.getSlotsSuccess({ slots: [] })),
    ));

    this.getRewardsStats$ = createEffect(() => this.actions$.pipe(
      ofType(BlockProductionOverviewActions.getRewardsStats, BlockProductionOverviewActions.close),
      this.latestActionState(),
      switchMap(({ action }) =>
        action.type === BlockProductionOverviewActions.close.type
          ? EMPTY
          : this.bpOverviewService.getRewardsAllTimeStats(),
      ),
      map(stats => BlockProductionOverviewActions.getRewardsStatsSuccess({ stats })),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, BlockProductionOverviewActions.getRewardsStatsSuccess(undefined)),
    ));
  }
}
