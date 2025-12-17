import { Data, Params } from '@angular/router';
import { RouterReducerState } from '@ngrx/router-store';

export interface MergedRoute {
  url: string;
  queryParams: Params;
  params: Params;
  data: Data;
}

export type MergedRouteReducerState = RouterReducerState<MergedRoute>;
