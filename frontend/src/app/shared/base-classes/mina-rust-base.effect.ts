import { FeatureAction, MinaBaseEffect } from '@openmina/shared';
import { MinaState } from '@app/app.setup';
import { OperatorFunction } from 'rxjs';
import { selectActionAndState, selectLatestStateSlice } from '@shared/constants/store-functions';
import { Selector, Store } from '@ngrx/store';

//todo: remove when no longer used anywhere
export abstract class MinaRustBaseEffect<A extends FeatureAction<any>> extends MinaBaseEffect<A, MinaState> {
}


export abstract class BaseEffect {
  protected readonly latestActionState = <Action>(): OperatorFunction<Action, { action: Action; state: MinaState }> =>
    selectActionAndState<Action>(this.store, this.selector);

  protected readonly latestActionStateSlice = <Slice extends object, Action>(path: string): OperatorFunction<Action, {
    action: Action;
    state: Slice
  }> => selectLatestStateSlice<Slice, Action>(this.store, this.selector, path);

  protected constructor(protected store: Store<MinaState>, private selector: Selector<MinaState, any>) {}
}
