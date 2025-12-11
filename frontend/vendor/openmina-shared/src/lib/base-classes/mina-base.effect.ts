import { Store, Action } from '@ngrx/store';
import { Selector } from '@ngrx/store/src/models';
import { OperatorFunction } from 'rxjs';
import { FeatureAction } from '../types/store/feature-action.type';
import { selectActionAndState, selectLatestStateSlice } from '../constants/store-functions';

export abstract class MinaBaseEffect<A extends FeatureAction<any>, State> {

  protected readonly latestActionState = <Action extends A>(): OperatorFunction<Action, { action: Action; state: State; }> => selectActionAndState<State, Action>(this.store, this.selector);
  protected readonly latestStateSlice = <Slice extends object, Action>(path: string): OperatorFunction<Action, {
    action: Action;
    state: Slice
  }> => selectLatestStateSlice<Slice, State, Action>(this.store, this.selector, path);

  protected constructor(protected store: Store<State>,
                        private selector: Selector<State, any>) { }
}
