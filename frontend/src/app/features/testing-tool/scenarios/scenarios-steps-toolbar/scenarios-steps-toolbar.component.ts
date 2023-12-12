import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  TestingToolScenariosCreateCluster,
  TestingToolScenariosStartScenario,
} from '@testing-tool/scenarios/testing-tool-scenarios.actions';
import {
  selectTestingToolScenariosClusterId,
  selectTestingToolScenariosScenario,
  selectTestingToolScenariosScenarioIsRunning,
} from '@testing-tool/scenarios/testing-tool-scenarios.state';
import { skip } from 'rxjs';
import { TestingToolScenario } from '@shared/types/testing-tool/scenarios/testing-tool-scenario.type';

@Component({
  selector: 'mina-scenarios-steps-toolbar',
  templateUrl: './scenarios-steps-toolbar.component.html',
  styleUrls: ['./scenarios-steps-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'fx-row-vert-cent h-xl pl-12' },
})
export class ScenariosStepsToolbarComponent extends StoreDispatcher implements OnInit {

  scenarioIsRunning: boolean = false;
  scenario: TestingToolScenario;

  private clusterId: string;

  constructor() {super();}

  ngOnInit(): void {
    this.listenToRunningScenario();
    this.listenToScenario();
    this.listenToCluster();
  }

  runSteps(): void {
    if (this.clusterId) {
      this.dispatch(TestingToolScenariosStartScenario);
    } else {
      this.dispatch(TestingToolScenariosCreateCluster);
    }
  }

  private listenToRunningScenario(): void {
    this.select(selectTestingToolScenariosScenarioIsRunning, (running: boolean) => {
      this.scenarioIsRunning = running;
      this.detect();
    }, skip(1));
  }

  private listenToScenario(): void {
    this.select(selectTestingToolScenariosScenario, (scenario: TestingToolScenario) => {
      this.scenario = scenario;
      this.detect();
    }, skip(1));
  }

  private listenToCluster(): void {
    this.select(selectTestingToolScenariosClusterId, (clusterId: string) => {
      this.clusterId = clusterId;
    });
  }
}
