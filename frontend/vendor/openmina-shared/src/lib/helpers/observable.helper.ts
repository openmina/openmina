import { MonoTypeOperatorFunction, pipe, tap } from 'rxjs';

export function log<T>(): MonoTypeOperatorFunction<T> {
  return pipe(tap(v => console.log(v)));
}
