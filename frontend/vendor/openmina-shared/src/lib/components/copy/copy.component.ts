import { ChangeDetectionStrategy, Component, HostBinding, Input, OnInit } from '@angular/core';
import { OpenminaEagerSharedModule } from '../../openmina-eager-shared.module';
import { CommonModule } from '@angular/common';
import { REQUIRED } from '../../constants/angular';

@Component({
    selector: 'mina-copy',
    templateUrl: './copy.component.html',
    styleUrls: ['./copy.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    imports: [OpenminaEagerSharedModule, CommonModule]
})
export class CopyComponent implements OnInit {

  @Input(REQUIRED) value: string;
  @Input() display: string;
  @Input() hidden: boolean = true;

  @HostBinding('class.no-hide') get isHidden(): boolean {
    return !this.hidden;
  }

  ngOnInit(): void {
    if (this.display === undefined) {
      this.display = this.value;
    }
  }

}
