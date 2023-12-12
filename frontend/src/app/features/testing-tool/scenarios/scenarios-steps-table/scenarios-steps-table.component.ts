import { ChangeDetectionStrategy, Component, OnInit, TemplateRef, ViewChild } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { SecDurationConfig, TableColumnList } from '@openmina/shared';
import { selectTestingToolScenariosScenario } from '@testing-tool/scenarios/testing-tool-scenarios.state';
import { TestingToolScenario } from '@shared/types/testing-tool/scenarios/testing-tool-scenario.type';
import { TestingToolScenarioStep } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-step.type';
import { filter } from 'rxjs';

@Component({
  selector: 'mina-scenarios-steps-table',
  templateUrl: './scenarios-steps-table.component.html',
  styleUrls: ['./scenarios-steps-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column' }
})
export class ScenariosStepsTableComponent extends MinaTableRustWrapper<TestingToolScenarioStep> implements OnInit {

  protected readonly tableHeads: TableColumnList<TestingToolScenarioStep> = [
    { name: '' },
    { name: '' },
    { name: 'kind' },
    { name: 'dialer' },
    { name: 'data' },
  ];

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToScenariosChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [30, 30, 110, 90, '1fr'];
    this.table.minWidth = 500;
  }

  private listenToScenariosChanges(): void {
    this.select(selectTestingToolScenariosScenario, (scenario: TestingToolScenario) => {
      this.table.rows = scenario.steps;
      this.table.detect();
      this.detect();
    }, filter(Boolean));
  }

  protected override onRowClick(row: TestingToolScenarioStep): void {
  }

}
