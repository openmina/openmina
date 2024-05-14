import { MinaState } from '@app/app.setup';
import { Directive, OnDestroy } from '@angular/core';
import { UntilDestroy } from '@ngneat/until-destroy';
import { BaseStoreDispatcher } from '@openmina/shared';
import { Action } from '@ngrx/store';

@UntilDestroy()
@Directive()
export abstract class StoreDispatcher extends BaseStoreDispatcher<MinaState> implements OnDestroy {

  protected dispatch2(action: Action): void {
    this.store.dispatch(action);
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
  }

}
