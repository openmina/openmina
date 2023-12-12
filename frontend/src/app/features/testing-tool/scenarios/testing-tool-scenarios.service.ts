import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { map, Observable, of, switchMap } from 'rxjs';
import { TestingToolScenario } from '@shared/types/testing-tool/scenarios/testing-tool-scenario.type';
import { TestingToolScenarioStep } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-step.type';
import { TestingToolScenarioEvent } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-event.type';

@Injectable({ providedIn: 'root' })
export class TestingToolScenariosService {

  readonly baseUrl = 'http://65.109.110.75:11000';

  constructor(private http: HttpClient) { }

  getScenario(): Observable<TestingToolScenario> {
    return this.http.get(this.baseUrl + '/scenarios').pipe(
      map((response: any) => response[0].id),
      switchMap((scenarioId: string) => this.getScenarioWithId(scenarioId)),
    );
    // return of({
    //   info: {
    //     id: '1',
    //     description: 'Scenario 1',
    //   },
    //   steps: [
    //     {
    //       // kind: string;
    //       // dialer?: number;
    //       // listener?: string;
    //       node_id: 13241,
    //       // event?: string;
    //       index: 1,
    //       kind: 'dialer',
    //       dialer: 1,
    //       listener: '2',
    //     },
    //     {
    //       index: 2,
    //       node_id: 56473,
    //       kind: 'dialer',
    //       dialer: 2,
    //       listener: '3',
    //     },
    //     {
    //       index: 3,
    //       node_id: 65474,
    //       kind: 'dialer',
    //       dialer: 3,
    //       listener: '4',
    //     },
    //   ],
    // });
  }

  private getScenarioWithId(id: string): Observable<TestingToolScenario> {
    return this.http.get<TestingToolScenario>(`${this.baseUrl}/scenarios/${id}`).pipe(
      map((scenario: TestingToolScenario) => {
        return {
          ...scenario,
          steps: scenario.steps.map((step: TestingToolScenarioStep, index: number) => ({
            ...step,
            index,
          })),
        };
      }),
    );
  }

  reloadScenario(clusterId: string, scenarioId: string): Observable<TestingToolScenario> {
    return this.http.post<TestingToolScenario>(`${this.baseUrl}/clusters/${clusterId}/scenarios/reload`, null).pipe(
      switchMap(() => this.getScenarioWithId(scenarioId)),
    )
  }

  addStep(scenarioId: string, step: any): Observable<void> {
    return this.http.put<void>(`${this.baseUrl}/scenarios/${scenarioId}/steps`, step);
  }

  createCluster(scenarioId: string): Observable<string> {
    return this.http.put<{ cluster_id: string }>(`${this.baseUrl}/clusters/create/${scenarioId}`, null).pipe(
      map(r => r.cluster_id),
    );
  }

  startScenario(clusterId: string): Observable<void> {
    return this.http.post<void>(`${this.baseUrl}/clusters/${clusterId}/run`, null);
  }

  getPendingEvents(clusterId: string): Observable<TestingToolScenarioEvent[]> {
    return this.http.get<any>(`${this.baseUrl}/clusters/${clusterId}/nodes/events/pending`).pipe(
      // return of([
      //   {
      //     'node_id': 0,
      //     'pending_events': [
      //       {
      //         'id': '10_1',
      //         'event': 'P2p, Connection, OfferSdpReady, 2awmjihpsd9TkdaqEkvrJ2UguAsoigRtgoiKei4AxpA5EVzr6JZ, Ok',
      //       },
      //       {
      //         'id': '253',
      //         'event': 'P2p, Connection, OfferSdpReady, 2awmjihpsd9TkdaqEkvrJ2UguAsoigRtgoiKei4AxpA5EVzr6JZ, Ok',
      //       },
      //       {
      //         'id': '423',
      //         'event': 'P2p, Connection, OfferSdpReady, 2awmjihpsd9TkdaqEkvrJ2UguAsoigRtgoiKei4AxpA5EVzr6JZ, Ok',
      //       },
      //     ],
      //   }]).pipe(
      map((response: any[]) =>
        response.reduce((acc, curr) => [
          ...acc,
          ...curr.pending_events.map((ev: any) => ({
            ...ev,
            node_id: curr.node_id,
          })),
        ], []),
      ),
    );
  }
}
