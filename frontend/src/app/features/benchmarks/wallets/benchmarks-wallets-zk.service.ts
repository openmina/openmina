import { Injectable } from '@angular/core';
import { BehaviorSubject, filter, Observable, of, ReplaySubject, switchMap } from 'rxjs';
import { BenchmarksZkapp } from '@shared/types/benchmarks/transactions/benchmarks-zkapp.type';
import { fromPromise } from 'rxjs/internal/observable/innerFrom';
import { CONFIG } from '@shared/constants/config';

@Injectable()
export class BenchmarksWalletsZkService {

  private readonly updates = new BehaviorSubject<string>(null);
  private readonly o1jsInterface: BehaviorSubject<any> = new BehaviorSubject<any>(null);

  readonly updates$ = this.updates.asObservable();

  loadO1js(): void {
    this.loadScript();
  }

  sendZkApp(zkApps: BenchmarksZkapp[]): Observable<any> {
    console.log('sendZkApp', zkApps);
    // return of([]);
    return this.o1jsInterface.pipe(
      filter(Boolean),
      switchMap((o1js: any) => {
        return fromPromise(o1js.sendZkApp(CONFIG.globalConfig?.graphQL, zkApps[0], this.updates));
      }),
    );
  }

  private loadScript(): void {
    const script = document.createElement('script');
    script.src = 'assets/o1js/main.js';
    script.onload = () => {
      if (typeof (window as any).$ !== 'undefined') {
        const $ = (window as any).$;
        console.log('Script loaded:', $);
        this.o1jsInterface.next($.default);
      } else {
        console.error('$ is not defined after loading the script');
      }
    };
    document.body.appendChild(script);
  }
}
