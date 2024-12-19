import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect, isDesktop, isMobile } from '@openmina/shared';
import { EMPTY, map, switchMap } from 'rxjs';
import { catchErrorAndRepeat2 } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Store } from '@ngrx/store';
import { BaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import { BlockProductionModule } from '@block-production/block-production.module';
import { BlockProductionWonSlotsService } from '@block-production/won-slots/block-production-won-slots.service';
import { BLOCK_PRODUCTION_WON_SLOTS_KEY, BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';
import { BlockProductionWonSlotsStatus } from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { fromPromise } from 'rxjs/internal/observable/innerFrom';

@Injectable({
  providedIn: BlockProductionModule,
})
export class BlockProductionWonSlotsEffects extends BaseEffect {

  readonly init$: Effect;
  readonly getSlots$: Effect;
  readonly setActiveSlotNumber$: Effect;

  constructor(private router: Router,
              private actions$: Actions,
              private wonSlotsService: BlockProductionWonSlotsService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(BlockProductionWonSlotsActions.init),
      map(() => BlockProductionWonSlotsActions.getSlots()),
    ));

    this.getSlots$ = createEffect(() => this.actions$.pipe(
      ofType(BlockProductionWonSlotsActions.getSlots, BlockProductionWonSlotsActions.close),
      this.latestActionState(),
      switchMap(({ action, state }) =>
        action.type === BlockProductionWonSlotsActions.close.type
          ? EMPTY
          : this.wonSlotsService.getSlots().pipe(
            switchMap(({ slots, epoch }) => {
              const bpState = state.blockProduction[BLOCK_PRODUCTION_WON_SLOTS_KEY];
              const activeSlotRoute = bpState.activeSlotRoute;
              let newActiveSlot = slots.find(s => s.globalSlot.toString() === activeSlotRoute);
              if (
                (isDesktop() && !activeSlotRoute)
                || (activeSlotRoute && !newActiveSlot)
                || (isMobile() && !activeSlotRoute && bpState.slots.length === 0)
              ) {
                newActiveSlot = slots.find(s => s.active)
                  ?? slots.find(s => s.status === BlockProductionWonSlotsStatus.Committed)
                  ?? slots.find(s => s.status === BlockProductionWonSlotsStatus.Scheduled)
                  ?? slots.find(s => !s.status);
              }
              const routes: string[] = [Routes.BLOCK_PRODUCTION, Routes.WON_SLOTS];
              if (
                newActiveSlot && isDesktop()
                || (activeSlotRoute && !bpState.activeSlot)
                || (activeSlotRoute && bpState.openSidePanel)
              ) {
                routes.push(newActiveSlot?.globalSlot.toString() ?? '');
              }
              return fromPromise(this.router.navigate(routes, { queryParamsHandling: 'merge' })).pipe(map(() => ({
                slots,
                epoch,
                activeSlot: newActiveSlot,
              })));
            }),
            map(data => BlockProductionWonSlotsActions.getSlotsSuccess(data)),
          ),
      ),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, BlockProductionWonSlotsActions.getSlotsSuccess({
        slots: [],
        epoch: undefined,
        activeSlot: undefined,
      })),
    ));

    this.setActiveSlotNumber$ = createEffect(() => this.actions$.pipe(
      ofType(BlockProductionWonSlotsActions.setActiveSlotNumber),
      this.latestActionState(),
      map(({ action, state }) => BlockProductionWonSlotsActions.setActiveSlot({
        slot: state.blockProduction[BLOCK_PRODUCTION_WON_SLOTS_KEY].slots.find(s => s.globalSlot === action.slotNumber),
      })),
    ));
  }
}
