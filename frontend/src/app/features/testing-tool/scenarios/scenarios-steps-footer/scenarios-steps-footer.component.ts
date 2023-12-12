import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { TestingToolScenariosAddStep } from '@testing-tool/scenarios/testing-tool-scenarios.actions';
import { selectTestingToolScenariosScenario } from '@testing-tool/scenarios/testing-tool-scenarios.state';
import { filter } from 'rxjs';
import { TestingToolScenario } from '@shared/types/testing-tool/scenarios/testing-tool-scenario.type';

@Component({
  selector: 'mina-scenarios-steps-footer',
  templateUrl: './scenarios-steps-footer.component.html',
  styleUrls: ['./scenarios-steps-footer.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column p-12' },
})
export class ScenariosStepsFooterComponent extends StoreDispatcher implements OnInit {

  stepIsAdding: boolean = false;
  stepCount: number = 0;

  ngOnInit(): void {
    this.listenToStepCount();
  }

  addStep(json: string): void {
    this.dispatch(TestingToolScenariosAddStep, { runScenario: true, step: JSON.parse(json) });
    this.stepIsAdding = false;
  }

  private listenToStepCount(): void {
    this.select(selectTestingToolScenariosScenario, (scenario: TestingToolScenario) => {
      this.stepCount = scenario.steps.length;
    }, filter(Boolean));
  }
}
