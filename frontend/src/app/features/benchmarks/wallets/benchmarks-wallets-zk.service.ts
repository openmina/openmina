import { Injectable } from '@angular/core';
import { BehaviorSubject, filter, map, Observable, switchMap } from 'rxjs';
import { BenchmarksZkapp } from '@shared/types/benchmarks/transactions/benchmarks-zkapp.type';
import { fromPromise } from 'rxjs/internal/observable/innerFrom';
import { CONFIG } from '@shared/constants/config';
import { any } from '@openmina/shared';

@Injectable()
export class BenchmarksWalletsZkService {

  private readonly updates = new BehaviorSubject<{ step: string, duration: number }>(null);
  private readonly o1jsInterface: BehaviorSubject<any> = new BehaviorSubject<any>(null);

  readonly updates$ = this.updates.asObservable();

  loadO1js(): void {
    this.loadScript();
  }

  sendZkApp(zkApps: BenchmarksZkapp[]): Observable<Partial<{
    zkApps: BenchmarksZkapp[],
    error: Error
  }>> {
    return this.o1jsInterface.pipe(
      filter(Boolean),
      switchMap((o1js: any) => {
        return fromPromise(o1js.updateZkApp(CONFIG.globalConfig?.graphQL, zkApps[0], this.updates));
      }),
      map((response: any) => {
        if (response.errors[0]) {
          let error = new Error(response.errors[0]);
          error.name = response.status;
          return { error, zkApps };
        }
        return { zkApps };
      }),
    );
  }

  private loadScript(): void {
    if (any(window).o1jsWrapper) {
      return;
    }
    const script = document.createElement('script');
    script.src = 'assets/o1js/o1jsWrapper.js';
    script.onload = () => {
      this.o1jsInterface.next(any(window).o1jsWrapper.default);
    };
    document.body.appendChild(script);
  }
}
