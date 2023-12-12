import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, HostListener, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-dashboard-node',
  templateUrl: './dashboard-node.component.html',
  styleUrls: ['./dashboard-node.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class DashboardNodeComponent extends StoreDispatcher implements AfterViewInit {

  barWidth: number = 0;

  @ViewChild('barsWrapper') parent: ElementRef<HTMLDivElement>;

  @HostListener('window:resize') onResize(): void {
    this.barWidth = this.parent.nativeElement.offsetWidth / 290;
  }

  ngAfterViewInit(): void {
    this.onResize();
    this.detect();
  }
}
