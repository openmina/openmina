import { inject, Injectable } from '@angular/core';
import { BehaviorSubject, catchError, filter, map, Observable, of, switchMap } from 'rxjs';
import { BenchmarksZkapp } from '@shared/types/benchmarks/transactions/benchmarks-zkapp.type';
import { fromPromise } from 'rxjs/internal/observable/innerFrom';
import { CONFIG } from '@shared/constants/config';
import { any } from '@openmina/shared';
import { DOCUMENT } from '@angular/common';

@Injectable()
export class BenchmarksWalletsZkService {

  private readonly updates = new BehaviorSubject<{ step: string, duration: number }>(null);
  private readonly o1jsInterface: BehaviorSubject<any> = new BehaviorSubject<any>(null);
  private readonly document: Document = inject(DOCUMENT);

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
        const executeSequentially = async (): Promise<any[]> => {
          const results: any[] = [];
          for (const zkApp of zkApps) {
            const response = await o1js.updateZkApp(CONFIG.globalConfig?.graphQL, zkApp, this.updates);
            results.push(response);
          }
          return results;
        };

        return fromPromise(executeSequentially());
      }),
      map((responses: any[]) => {
        const errors = responses.filter(response => response.errors && response.errors[0]);
        if (errors.length > 0) {
          let error = new Error(errors[0].errors[0]);
          error.name = errors[0].status;
          return { error, zkApps };
        }
        return { zkApps };
      }),
      catchError((error: Error) => {
        return of({ error, zkApps });
      }),
    );
  }

  private loadScript(): void {
    if (any(window).o1jsWrapper) {
      return;
    }
    const script = this.document.createElement('script');
    script.src = 'assets/o1js/dist/o1js-wrapper.js';
    script.onload = () => {
      this.o1jsInterface.next(any(window).o1jsWrapper.default);
    };
    this.document.body.appendChild(script);
  }
}
