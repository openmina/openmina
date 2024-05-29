import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect } from '@openmina/shared';
import { EMPTY, map, switchMap } from 'rxjs';
import { catchErrorAndRepeat2 } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Store } from '@ngrx/store';
import { BaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import { BlockProductionModule } from '@block-production/block-production.module';
import { BlockProductionWonSlotsService } from '@block-production/won-slots/block-production-won-slots.service';
import { BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';
import {
  BlockProductionWonSlotsSlot,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import {
  BlockProductionWonSlotsEpoch,
} from '@shared/types/block-production/won-slots/block-production-won-slots-epoch.type';

@Injectable({
  providedIn: BlockProductionModule,
})
export class BlockProductionWonSlotsEffects extends BaseEffect {

  readonly getSlots$: Effect;

  constructor(private actions$: Actions,
              private wonSlotsService: BlockProductionWonSlotsService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getSlots$ = createEffect(() => this.actions$.pipe(
      ofType(BlockProductionWonSlotsActions.getSlots, BlockProductionWonSlotsActions.close),
      this.latestActionState(),
      switchMap(({ action }) =>
        action.type === BlockProductionWonSlotsActions.close.type
          ? EMPTY
          : this.wonSlotsService.getSlots().pipe(
            map(({ slots, epoch }: {
              slots: BlockProductionWonSlotsSlot[],
              epoch: BlockProductionWonSlotsEpoch
            }) => BlockProductionWonSlotsActions.getSlotsSuccess({ slots, epoch })),
          ),
      ),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, BlockProductionWonSlotsActions.getSlotsSuccess({
        slots: [],
        epoch: undefined,
      })),
    ));
  }
}
