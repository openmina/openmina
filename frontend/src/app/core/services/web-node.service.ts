import { Inject, Injectable } from '@angular/core';
import {
  BehaviorSubject,
  catchError,
  EMPTY,
  filter,
  from,
  fromEvent,
  map,
  merge,
  Observable,
  of,
  switchMap,
  tap,
} from 'rxjs';
import base from 'base-x';
import { any } from '@openmina/shared';
import { HttpClient } from '@angular/common/http';
import { sendSentryEvent } from '@shared/helpers/webnode.helper';
import { DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { DOCUMENT } from '@angular/common';
import { FileProgressHelper } from '@core/helpers/file-progress.helper';

@Injectable({
  providedIn: 'root',
})
export class WebNodeService {

  private readonly webnode$: BehaviorSubject<any> = new BehaviorSubject<any>(null);
  private webNodeKeyPair: { publicKey: string, privateKey: string };
  private webNodeStartTime: number;
  private sentryEvents: any = {};

  readonly webnodeProgress$: BehaviorSubject<string> = new BehaviorSubject<string>('');

  constructor(private http: HttpClient,
              @Inject(DOCUMENT) private document: Document) {
    FileProgressHelper.initDownloadProgress();
    const basex = base('123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz');
    any(window)['bs58btc'] = {
      encode: (buffer: Uint8Array | number[]) => 'z' + basex.encode(buffer),
      decode: (string: string) => basex.decode(string.substring(1)),
    };
  }

  private loadWebnodeJs(): void {
    if (this.document.querySelector('[data-webnode]')) {
      return;
    }

    const script = this.document.createElement('script');
    script.type = 'module';
    script.setAttribute('data-webnode', 'true');
    script.textContent = `
      import('./assets/webnode/pkg/openmina_node_web.js')
        .then(v => {
          window.webnode = v;
          window.dispatchEvent(new CustomEvent('webNodeLoaded'));
        })
        .catch(er => {
          if (window.env?.configs.some(c => c.isWebNode)) {
            console.error('Failed to load Web Node:', er);
          }
        });
    `;

    this.document.body.appendChild(script);
  }

  loadWasm$(): Observable<void> {
    this.webNodeStartTime = Date.now();
    // this.loadWebnodeJs();
    return merge(
      of(any(window).webnode).pipe(filter(Boolean)),
      fromEvent(window, 'webNodeLoaded'),
    ).pipe(
      switchMap(() => this.http.get<{ publicKey: string, privateKey: string }>('assets/webnode/web-node-secrets.json')),
      tap(data => {
        this.webNodeKeyPair = data;
      }),
      map(() => void 0),
    );
  }

  startWasm$(): Observable<any> {
    return of(any(window).webnode)
      .pipe(
        switchMap((wasm: any) => from(wasm.default('assets/webnode/pkg/openmina_node_web_bg.wasm')).pipe(map(() => wasm))),
        switchMap((wasm) => {
          this.webnodeProgress$.next('Loaded');
          return from(wasm.run(this.webNodeKeyPair.privateKey));
        }),
        tap((webnode: any) => {
          (window as any)['webnode'] = webnode;
          this.webnode$.next(webnode);
          this.webnodeProgress$.next('Started');
        }),
        catchError((error) => {
          sendSentryEvent('WebNode failed to start: ' + error.message);
          console.error(error);
          return of(null);
        }),
        switchMap(() => this.webnode$.asObservable()),
        filter(Boolean),
      );
  }

  get status$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).status())),
    );
  }

  get blockProducerStats$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).stats().block_producer())),
    );
  }

  get peers$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from(any(handle).state().peers())),
      tap((peers) => {
        // if (!this.sentryEvents.sentNoPeersEvent && Date.now() - this.webNodeStartTime >= 5000 && peers.length === 0) {
        //   sendSentryEvent('WebNode has no peers after 5 seconds from startup.');
        //   this.sentryEvents.sentNoPeersEvent = true;
        // }
        // if (!this.sentryEvents.sentPeersEvent && peers.length > 0) {
        //   this.sentryEvents.sentPeersEvent = true;
        // }
        if (!this.sentryEvents.firstPeerConnected && peers.some((p: any) => p.connection_status === DashboardPeerStatus.CONNECTED)) {
          const seconds = (Date.now() - this.webNodeStartTime) / 1000;
          sendSentryEvent(`WebNode connected in ${seconds.toFixed(1)}s`);
          this.sentryEvents.firstPeerConnected = true;
          this.webnodeProgress$.next('Connected');
        }
      }),
    );
  }

  get messageProgress$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).state().message_progress())),
    );
  }

  get sync$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).stats().sync())),
    );
  }

  get accounts$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).ledger().latest().accounts().all())),
    );
  }

  get bestChainUserCommands$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).transition_frontier().best_chain().user_commands())),
    );
  }

  sendPayment$(payment: any): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).transaction_pool().inject().payment(payment))),
    );
  }

  get transactionPool$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(handle => from((handle as any).transaction_pool().get())),
    );
  }
}
