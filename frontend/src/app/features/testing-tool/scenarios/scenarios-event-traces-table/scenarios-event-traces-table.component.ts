import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  selectTestingToolScenariosPendingEvents,
  selectTestingToolScenariosScenarioIsRunning,
} from '@testing-tool/scenarios/testing-tool-scenarios.state';
import { TestingToolScenarioEvent } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-event.type';
import { TestingToolScenariosAddStep } from '@testing-tool/scenarios/testing-tool-scenarios.actions';
import { skip } from 'rxjs';

@Component({
  selector: 'mina-scenarios-event-traces-table',
  templateUrl: './scenarios-event-traces-table.component.html',
  styleUrls: ['./scenarios-event-traces-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class ScenariosEventTracesTableComponent extends StoreDispatcher implements OnInit {

  events: TestingToolScenarioEvent[];
  scenarioIsRunning: boolean = false;

  readonly eventsTrackBy = (index: number, event: TestingToolScenarioEvent) => event.id + event.details;

  constructor() {super();}

  ngOnInit(): void {
    this.listenToEvents();
    this.listenToRunningScenario();
  }

  addEventToSteps(event: TestingToolScenarioEvent): void {
    this.dispatch(TestingToolScenariosAddStep, {
      runScenario: true,
      step: {
        kind: 'Event',
        node_id: event.node_id,
        event: event.event,
      },
    });
  }

  private listenToEvents(): void {
    this.select(selectTestingToolScenariosPendingEvents, (events: TestingToolScenarioEvent[]) => {
      this.events = events;
      this.detect();
    });
  }

  private listenToRunningScenario(): void {
    this.select(selectTestingToolScenariosScenarioIsRunning, (running: boolean) => {
      this.scenarioIsRunning = running;
      this.detect();
    }, skip(1));
  }
}
