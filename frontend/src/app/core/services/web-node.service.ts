import { Injectable } from '@angular/core';
import { BehaviorSubject, filter, from, fromEvent, map, merge, Observable, of, switchMap, tap } from 'rxjs';
import base from 'base-x';
import { any, log } from '@openmina/shared';
import { HttpClient } from '@angular/common/http';

@Injectable({
  providedIn: 'root',
})
export class WebNodeService {

  private readonly backendSubject$: BehaviorSubject<any> = new BehaviorSubject<any>(null);
  private backend: any;
  private webNodeKeyPair: { publicKey: string, privateKey: string };
  webNodeState: string = 'notLoaded';

  constructor(private http: HttpClient) {
    const basex = base('123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz');
    any(window)['bs58btc'] = {
      encode: (buffer: Uint8Array | number[]) => 'z' + basex.encode(buffer),
      decode: (string: string) => basex.decode(string.substring(1)),
    };
  }

  loadWasm$(): Observable<void> {
    console.log('---LOADING WEBNODE---');
    return merge(
      of(any(window).webnode).pipe(filter(Boolean)),
      fromEvent(window, 'webNodeLoaded'),
    ).pipe(
      switchMap(() => this.http.get<{ publicKey: string, privateKey: string }>('assets/webnode/web-node-secrets.json')),
      tap(data => this.webNodeKeyPair = data),
      map(() => void 0),
    );
  }

  startWasm$(): Observable<any> {
    console.log('---STARTING WEBNODE---');
    return of(any(window).webnode)
      .pipe(
        switchMap((wasm: any) => from(wasm.default('assets/webnode/pkg/openmina_node_web_bg.wasm')).pipe(map(() => wasm))),
        switchMap((wasm) => {
          console.log(wasm);
          return from(wasm.run(this.webNodeKeyPair.privateKey));
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

  get webNodeKeys(): { publicKey: string, privateKey: string } {
    return this.webNodeKeyPair;
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

  get accounts$(): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).ledger().latest().accounts().all())),
    );
  }

  get bestChainUserCommands$(): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).transition_frontier().best_chain().user_commands())),
    );
  }

  sendPayment$(payment: any): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).transaction_pool().inject().payment(payment))),
    );
  }

  get transactionPool$(): Observable<any> {
    return this.backendSubject$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).transaction_pool().get())),
    );
  }
}
