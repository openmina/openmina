import { ChangeDetectionStrategy, Component } from '@angular/core';

@Component({
  selector: 'mina-testing-tool',
  templateUrl: './testing-tool.component.html',
  styleUrls: ['./testing-tool.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class TestingToolComponent {

}
