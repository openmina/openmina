import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect } from '@openmina/shared';
import { EMPTY, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Store } from '@ngrx/store';
import { Router } from '@angular/router';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  BLOCK_PRODUCTION_OVERVIEW_CLOSE,
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS,
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS,
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS,
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS,
  BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS,
  BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS,
  BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS,
  BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS,
  BlockProductionOverviewActions,
  BlockProductionOverviewClose,
  BlockProductionOverviewGetEpochDetails,
  BlockProductionOverviewGetEpochs,
  BlockProductionOverviewGetRewardsStats,
  BlockProductionOverviewGetSlots,
} from '@block-production/overview/block-production-overview.actions';
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
export class BlockProductionOverviewEffects extends MinaRustBaseEffect<BlockProductionOverviewActions> {

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
      ofType(BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS, BLOCK_PRODUCTION_OVERVIEW_CLOSE),
      this.latestActionState<BlockProductionOverviewGetEpochDetails | BlockProductionOverviewClose>(),
      switchMap(({ action }) =>
        action.type === BLOCK_PRODUCTION_OVERVIEW_CLOSE
          ? EMPTY
          : this.bpOverviewService.getEpochDetails(action.payload),
      ),
      tap(response => this.router.navigate([Routes.BLOCK_PRODUCTION, Routes.OVERVIEW, response.epochNumber], { queryParamsHandling: 'merge' })),
      switchMap((payload: BlockProductionOverviewEpochDetails) => [
        { type: BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS, payload },
        { type: BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS, payload: payload.epochNumber },
        { type: BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS, payload: payload.epochNumber },
      ]),
      catchErrorAndRepeat(MinaErrorType.GENERIC, BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS, undefined),
    ));

    this.getEpochs$ = createEffect(() => this.actions$.pipe(
      ofType(BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS, BLOCK_PRODUCTION_OVERVIEW_CLOSE),
      this.latestActionState<BlockProductionOverviewGetEpochs | BlockProductionOverviewClose>(),
      switchMap(({ action }) =>
        action.type === BLOCK_PRODUCTION_OVERVIEW_CLOSE
          ? EMPTY
          : this.bpOverviewService.getEpochs(action.payload),
      ),
      map((payload: BlockProductionOverviewEpoch[]) => ({
        type: BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS,
        payload,
      })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS, undefined),
    ));

    this.getActiveEpochSlots$ = createEffect(() => this.actions$.pipe(
      ofType(BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS, BLOCK_PRODUCTION_OVERVIEW_CLOSE),
      this.latestActionState<BlockProductionOverviewGetSlots | BlockProductionOverviewClose>(),
      switchMap(({ state, action }) =>
        action.type === BLOCK_PRODUCTION_OVERVIEW_CLOSE
          ? EMPTY
          : this.bpOverviewService.getSlots(state.blockProduction.overview.activeEpochNumber).pipe(
            map((payload: BlockProductionSlot[]) => ({ type: BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS, payload })),
          ),
      ),
      catchErrorAndRepeat(MinaErrorType.GENERIC, BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS, []),
    ));

    this.getRewardsStats$ = createEffect(() => this.actions$.pipe(
      ofType(BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS, BLOCK_PRODUCTION_OVERVIEW_CLOSE),
      this.latestActionState<BlockProductionOverviewGetRewardsStats | BlockProductionOverviewClose>(),
      switchMap(({ action }) =>
        action.type === BLOCK_PRODUCTION_OVERVIEW_CLOSE
          ? EMPTY
          : this.bpOverviewService.getRewardsAllTimeStats(),
      ),
      map(payload => ({ type: BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS, undefined),
    ));
  }
}
