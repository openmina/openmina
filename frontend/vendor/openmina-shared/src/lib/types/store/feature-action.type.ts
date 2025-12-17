import { Action } from '@ngrx/store';

export type FeatureAction<T extends string> = Action<T>;
