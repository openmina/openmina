import { MinaState } from '@app/app.setup';
import { Directive, OnDestroy } from '@angular/core';
import { UntilDestroy } from '@ngneat/until-destroy';
import { BaseStoreDispatcher } from '@openmina/shared';

@UntilDestroy()
@Directive()
export abstract class StoreDispatcher extends BaseStoreDispatcher<MinaState> implements OnDestroy {

  override ngOnDestroy(): void {
    super.ngOnDestroy();
  }

}
