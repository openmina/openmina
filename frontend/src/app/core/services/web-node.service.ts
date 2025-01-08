import { Injectable } from '@angular/core';
import { BehaviorSubject, catchError, EMPTY, filter, from, fromEvent, map, merge, Observable, of, switchMap, tap, throwError } from 'rxjs';
import base from 'base-x';
import { any, isBrowser, safelyExecuteInBrowser } from '@openmina/shared';
import { HttpClient } from '@angular/common/http';
import { sendSentryEvent } from '@shared/helpers/webnode.helper';
import { DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { FileProgressHelper } from '@core/helpers/file-progress.helper';
import { CONFIG } from '@shared/constants/config';

export interface PrivateStake {
  publicKey: string;
  password: string;
  stake: string;
}

@Injectable({
  providedIn: 'root',
})
export class WebNodeService {

  private readonly webnode$: BehaviorSubject<any> = new BehaviorSubject<any>(null);
  private readonly wasm$: BehaviorSubject<any> = new BehaviorSubject<any>(null);

  private webNodeKeyPair: { publicKey: string, privateKey: string };
  private webNodeNetwork: String;
  private webNodeStartTime: number;
  private sentryEvents: any = {};

  readonly webnodeProgress$: BehaviorSubject<string> = new BehaviorSubject<string>('');

  memory: WebAssembly.MemoryDescriptor;
  privateStake: PrivateStake;
  noBlockProduction: boolean = false;

  constructor(private http: HttpClient) {
    FileProgressHelper.initDownloadProgress();
    const basex = base('123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz');
    safelyExecuteInBrowser(() => {
      any(window).bs58btc = {
        encode: (buffer: Uint8Array | number[]) => 'z' + basex.encode(buffer),
        decode: (string: string) => basex.decode(string.substring(1)),
      };
    });
  }

  hasWebNodeConfig(): boolean {
    return CONFIG.configs.some(c => c.isWebNode);
  }

  isWebNodeLoaded(): boolean {
    if (isBrowser()) {
      return !!any(window).webnode;
    }
    return false;
  }

  loadWasm$(): Observable<void> {
    this.webNodeStartTime = Date.now();

    if (isBrowser()) {
      const args = (() => {
        const raw = localStorage.getItem('webnodeArgs');
        if (raw === null) {
          return null;
        }
        return JSON.parse(atob(raw));
      })();
      return merge(
        of(any(window).webnode).pipe(filter(Boolean)),
        fromEvent(window, 'webNodeLoaded'),
      ).pipe(
        switchMap(() => {
          const DEFAULT_NETWORK = 'devnet';
          if (!args) {
            return this.http.get<{ publicKey: string, privateKey: string }>('assets/webnode/web-node-secrets.json')
              .pipe(map(blockProducer => ({ blockProducer, network: DEFAULT_NETWORK })));
          }
          const data = { network: args['network'] || DEFAULT_NETWORK, blockProducer: {} as any };
          if (!!args['block_producer']) {
            data['blockProducer'] = {
              privateKey: args['block_producer'].sec_key,
              publicKey: args['block_producer'].pub_key,
            };
          }
          return of(data);
        }),
        tap(data => {
          this.webNodeKeyPair = data.blockProducer;
          this.webNodeNetwork = data.network;
        }),
        map(() => void 0),
      );
    }
    return EMPTY;
  }

  startWasm$(): Observable<any> {
    if (isBrowser()) {
      return of(any(window).webnode)
        .pipe(
          switchMap((wasm: any) => {
            this.wasm$.next(wasm);
            return from(wasm.default(undefined, new WebAssembly.Memory(this.memory)))
              .pipe(map(() => wasm));
          }),
          switchMap((wasm) => {
            this.webnodeProgress$.next('Loaded');
            const urls = (() => {
              if (typeof this.webNodeNetwork === 'number') {
                const url = `${window.location.origin}/clusters/${this.webNodeNetwork}/`;
                return {
                  seeds: url + 'seeds',
                  genesisConfig: url + 'genesis/config',
                };
              } else {
                return {
                  seeds: 'https://bootnodes.minaprotocol.com/networks/devnet-webrtc.txt',
                };
              }
            })();
            console.log('webnode config:', !!this.webNodeKeyPair.privateKey, this.webNodeNetwork, urls);
            let privateKey = this.privateStake ? [this.privateStake.stake, this.privateStake.password] : this.webNodeKeyPair.privateKey;
            if (this.noBlockProduction) {
              privateKey = null;
            }

            return from(wasm.run(privateKey, urls.seeds, urls.genesisConfig));
          }),
          tap((webnode: any) => {
            any(window).webnode = webnode;
            this.webnode$.next(webnode);
            this.webnodeProgress$.next('Started');

          }),
          catchError((error) => {
            sendSentryEvent('WebNode failed to start: ' + error.message);
            return throwError(() => new Error(error.message));
          }),
          switchMap(() => this.webnode$.asObservable()),
        );
    }
    return EMPTY;
  }

  get status$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).status())),
    );
  }

  get blockProducerStats$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).stats().block_producer())),
    );
  }

  get peers$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).state().peers())),
      tap(peers => {
        // if (!this.sentryEvents.sentNoPeersEvent && Date.now() - this.webNodeStartTime >= 5000 && peers.length === 0) {
        //   sendSentryEvent('WebNode has no peers after 5 seconds from startup.');
        //   this.sentryEvents.sentNoPeersEvent = true;
        // }
        // if (!this.sentryEvents.sentPeersEvent && peers.length > 0) {
        //   this.sentryEvents.sentPeersEvent = true;
        // }
        if (!this.sentryEvents.firstPeerConnected && peers.some((p: any) => p.connection_status === DashboardPeerStatus.CONNECTED)) {
          const seconds = (Date.now() - this.webNodeStartTime) / 1000;
          sendSentryEvent(`WebNode connected in ${seconds.toFixed(1)}s`, 'info');
          this.sentryEvents.firstPeerConnected = true;
          this.webnodeProgress$.next('Connected');
        }
      }),
    );
  }

  get messageProgress$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).state().message_progress())),
    );
  }

  get sync$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).stats().sync())),
    );
  }

  get accounts$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).ledger().latest().accounts().all())),
    );
  }

  get bestChainUserCommands$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).transition_frontier().best_chain().user_commands())),
    );
  }

  sendPayment$(payment: any): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).transaction_pool().inject().payment(payment))),
    );
  }

  get transactionPool$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => from(any(webnode).transaction_pool().get())),
    );
  }

  get envBuildDetails$(): Observable<any> {
    return this.wasm$.asObservable().pipe(
      filter(Boolean),
      map(webnode => webnode.build_env()),
    );
  }

  actions$(path: string): Observable<any> {
    let slot: string | number = path.split('?id=')[1];
    if (!isNaN(Number(slot))) {
      slot = Number(slot);
    }
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode => webnode.stats().actions(slot)),
    );
  }
}
