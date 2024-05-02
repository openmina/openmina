import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, map, switchMap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  MEMORY_RESOURCES_CLOSE,
  MEMORY_RESOURCES_GET,
  MEMORY_RESOURCES_GET_SUCCESS,
  MEMORY_RESOURCES_SET_GRANULARITY,
  MemoryResourcesActions,
  MemoryResourcesClose,
  MemoryResourcesGet,
} from '@resources/memory/memory-resources.actions';
import { MemoryResourcesService } from '@resources/memory/memory-resources.service';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';

@Injectable()
export class MemoryResourcesEffects extends MinaRustBaseEffect<MemoryResourcesActions> {

  readonly getResources$: Effect;
  readonly setGranularity$: Effect;

  constructor(private actions$: Actions,
              private memoryResourcesService: MemoryResourcesService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getResources$ = createEffect(() => this.actions$.pipe(
      ofType(MEMORY_RESOURCES_GET, MEMORY_RESOURCES_CLOSE),
      this.latestActionState<MemoryResourcesGet | MemoryResourcesClose>(),
      switchMap(({ action, state }) =>
        action.type === MEMORY_RESOURCES_CLOSE
          ? EMPTY
          : this.memoryResourcesService.getStorageResources(state.resources.memory.granularity),
      ),
      map((payload: MemoryResource) => ({ type: MEMORY_RESOURCES_GET_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, MEMORY_RESOURCES_GET_SUCCESS, undefined),
    ));

    this.setGranularity$ = createEffect(() => this.actions$.pipe(
      ofType(MEMORY_RESOURCES_SET_GRANULARITY),
      map((payload: MemoryResource) => ({ type: MEMORY_RESOURCES_GET, payload })),
    ));
  }
}
