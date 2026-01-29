import { Injectable } from '@angular/core';
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
  throwError,
  timer,
} from 'rxjs';
import base from 'base-x';
import {
  any,
  isBrowser,
  safelyExecuteInBrowser,
  getLocalStorage,
} from '@mina-rust/shared';
import { DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { FileProgressHelper } from '@core/helpers/file-progress.helper';
import { CONFIG } from '@shared/constants/config';

export interface PrivateStake {
  publicKey: string;
  password: string | null;
  stake: string;
}

export type BlockProducerConfig =
  | { mode: 'observer' }
  | { mode: 'uploaded'; data: PrivateStake }
  | { mode: 'auto' };

export type WebNodeConfiguration = {
  network: string;
  blockProducer: {
    publicKey: string;
    privateKey: string | [string, string] | null;
  };
};

@Injectable({
  providedIn: 'root',
})
export class WebNodeService {
  private readonly webnode$: BehaviorSubject<any> = new BehaviorSubject<any>(
    null,
  );
  private readonly wasm$: BehaviorSubject<any> = new BehaviorSubject<any>(null);

  private webNodeStartTime: number;
  private firstPeerConnected: boolean = false;

  readonly webnodeProgress$: BehaviorSubject<string> =
    new BehaviorSubject<string>('');

  memory: WebAssembly.MemoryDescriptor;
  blockProducerConfig: BlockProducerConfig = { mode: 'auto' };

  constructor() {
    FileProgressHelper.initDownloadProgress();
    const basex = base(
      '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz',
    );
    safelyExecuteInBrowser(() => {
      any(window).bs58btc = {
        encode: (buffer: Uint8Array | number[]) => 'z' + basex.encode(buffer),
        decode: (string: string) => basex.decode(string.substring(1)),
      };
    });
  }

  get publicKey(): string {
    return this.blockProducerConfig.mode === 'uploaded'
      ? this.blockProducerConfig.data.publicKey
      : undefined;
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

  loadWasm$(): Observable<WebNodeConfiguration> {
    this.webNodeStartTime = Date.now();

    if (isBrowser()) {
      return merge(
        of(any(window).webnode).pipe(filter(Boolean)),
        fromEvent(window, 'webNodeLoaded'),
      ).pipe(switchMap(() => this.getWebNodeConfiguration$()));
    }
    return EMPTY;
  }

  private getWebNodeConfiguration$(): Observable<WebNodeConfiguration> {
    const DEFAULT_NETWORK = 'devnet';

    switch (this.blockProducerConfig.mode) {
      case 'uploaded':
        console.log('WebNode: Using uploaded key configuration');
        return of({
          network: DEFAULT_NETWORK,
          blockProducer: {
            publicKey: this.blockProducerConfig.data.publicKey,
            privateKey: this.blockProducerConfig.data.password
              ? [
                  this.blockProducerConfig.data.stake,
                  this.blockProducerConfig.data.password,
                ]
              : this.blockProducerConfig.data.stake,
          },
        });

      case 'observer':
        console.log('WebNode: Running in observer mode (no block production)');
        return of({
          network: DEFAULT_NETWORK,
          blockProducer: {
            publicKey: '',
            privateKey: null,
          },
        });

      case 'auto':
        // Check localStorage for URL parameter args
        const args = this.getWebnodeArgsFromStorage();
        if (args) {
          console.log(
            'WebNode: Using webnodeArgs from localStorage (URL parameter)',
          );
          return of({
            network: args.network || DEFAULT_NETWORK,
            blockProducer: args.blockProducer || {
              publicKey: '',
              privateKey: null,
            },
          });
        }

        // No configuration found - start in observer mode
        console.log(
          'WebNode: No configuration found - starting in observer mode',
        );
        return of({
          blockProducer: {
            publicKey: '',
            privateKey: null,
          },
          network: DEFAULT_NETWORK,
        });
    }
  }

  private getWebnodeArgsFromStorage(): any | null {
    // localStorage value is set in ../../app.component.ts
    // from URL query param `a`.
    const raw = getLocalStorage()?.getItem('webnodeArgs');
    if (raw === null) {
      return null;
    }
    try {
      return JSON.parse(atob(raw));
    } catch (error) {
      console.error(
        'WebNode: Failed to parse webnodeArgs from localStorage:',
        error,
      );
      return null;
    }
  }

  startWasm$(config: WebNodeConfiguration): Observable<any> {
    if (isBrowser()) {
      return of(any(window).webnode).pipe(
        switchMap((wasm: any) => {
          this.wasm$.next(wasm);

          // nb: this is RUSTFLAGS "-Clink-args=--max-memory=4294967296" (4GiB)
          // nb: in .cargo/config.toml for the wasm32 target the div by 65536
          // nb: is because the WASM web API requires memory size to be specified
          // nb: in terms of 64KiB *pages*
          // todo: move to wherever angular injects `this.memory`
          this.memory.maximum = 4294967296 / 65536;
          return from(
            wasm.default(undefined, new WebAssembly.Memory(this.memory)),
          ).pipe(map(() => wasm));
        }),
        switchMap(wasm => {
          this.webnodeProgress$.next('Loaded');
          const urls = {
            seedUrls: CONFIG.globalConfig.webNodeSeedUrls,
            fixedSeeds: CONFIG.globalConfig.webNodeBootNodes,
          };
          let privateKey = config.blockProducer.privateKey;
          console.log(
            'webnode config: has private key?',
            !!privateKey,
            'seed urls?',
            urls,
          );

          return from(
            wasm.run(privateKey, urls.seedUrls, urls.fixedSeeds, null),
          );
        }),
        tap((webnode: any) => {
          any(window).webnode = webnode;
          this.webnode$.next(webnode);
          this.webnodeProgress$.next('Started');
        }),
        catchError(error => {
          console.error('WebNode failed to start:', error.message);
          return throwError(() => new Error(error.message));
        }),
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
      tap((peers: any) => {
        if (
          !this.firstPeerConnected &&
          peers.some(
            (p: any) => p.connection_status === DashboardPeerStatus.CONNECTED,
          )
        ) {
          this.firstPeerConnected = true;
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
      switchMap(webnode =>
        from(any(webnode).ledger().latest().accounts().all()),
      ),
    );
  }

  get bestChainUserCommands$(): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode =>
        from(any(webnode).transition_frontier().best_chain().user_commands()),
      ),
    );
  }

  sendPayment$(payment: any): Observable<any> {
    return this.webnode$.asObservable().pipe(
      filter(Boolean),
      switchMap(webnode =>
        from(any(webnode).transaction_pool().inject().payment(payment)),
      ),
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
