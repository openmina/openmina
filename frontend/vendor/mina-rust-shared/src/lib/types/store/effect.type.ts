import { Observable } from 'rxjs';
import { Action } from '@ngrx/store';
import { CreateEffectMetadata } from '@ngrx/effects';

export type Effect<T = Action> = Observable<T> & CreateEffectMetadata;
export type NonDispatchableEffect<T = Action, S = any> = Observable<{ action: T, state: S }> & CreateEffectMetadata;
