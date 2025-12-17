import { ChangeDetectionStrategy, Component, Input } from '@angular/core';
import { OpenminaSharedModule } from '../../../openmina-shared.module';
import { ManualDetection } from '../../../base-classes/manual-detection.class';
import { CommonModule } from '@angular/common';
import { REQUIRED } from '../../../constants/angular';

@Component({
    selector: 'mina-flame-time-graph-tooltip',
    imports: [OpenminaSharedModule, CommonModule],
    templateUrl: './flame-time-graph-tooltip.component.html',
    styleUrls: ['./flame-time-graph-tooltip.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    host: { class: 'bg-surface-top pt-5 f-10 flex-column border-rad-6' }
})
export class FlameTimeGraphTooltipComponent extends ManualDetection {

  @Input(REQUIRED) xSteps: string[];
  @Input(REQUIRED) activeXPointIndex: number;
  @Input(REQUIRED) range: string;
  @Input(REQUIRED) mean: number;
  @Input(REQUIRED) max: number;
  @Input(REQUIRED) calls: number;
  @Input(REQUIRED) totalTime: number;

}
