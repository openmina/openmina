import { Injectable } from '@angular/core';
import { BehaviorSubject, filter, from, fromEvent, map, Observable, of, switchMap, tap } from 'rxjs';
import base from 'base-x';
import { any, log } from '@openmina/shared';
import { CONFIG } from '@shared/constants/config';

@Injectable({
  providedIn: 'root',
})
export class WebNodeService {

  private readonly backendSubject$: BehaviorSubject<any> = new BehaviorSubject<any>(null);
  private backend: any;
  webNodeState: string = 'notLoaded';

  constructor() {
    const basex = base('123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz');
    any(window)['bs58btc'] = {
      encode: (buffer: Uint8Array | number[]) => 'z' + basex.encode(buffer),
      decode: (string: string) => basex.decode(string.substring(1)),
    };
  }

  loadWasm$(): Observable<void> {
    if ((window as any).webnode) {
      return of(void 0);
    }
    return fromEvent(window, 'webNodeLoaded').pipe(map(() => void 0));
  }

  startWasm$(): Observable<any> {
    return of((window as any).webnode)
      .pipe(
        switchMap((wasm: any) => from(wasm.default('assets/webnode/pkg/openmina_node_web_bg.wasm')).pipe(map(() => wasm))),
        switchMap((wasm) => {
          console.log(wasm);
          return from(wasm.run(CONFIG.webNodeKey));
        }),
        tap((jsHandle: any) => {
          this.backend = jsHandle;
          console.log('----------------WEBNODE----------------');
          console.log(jsHandle);
          this.backendSubject$.next(jsHandle);
        }),
        switchMap(() => this.backendSubject$.asObservable()),
        filter(Boolean),
      );
  }

  get status$(): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).status())),
    );
  }

  get blockProducerStats$(): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).stats().block_producer())),
    );
  }

  get peers$(): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from(any(handle).state().peers())),
    );
  }

  get messageProgress$(): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).state().message_progress())),
    );
  }

  get sync$(): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).stats().sync())),
    );
  }
}
