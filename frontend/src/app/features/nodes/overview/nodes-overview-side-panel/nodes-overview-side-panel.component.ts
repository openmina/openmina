import { ChangeDetectionStrategy, Component, ElementRef, NgZone, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectNodesOverviewActiveNode } from '@nodes/overview/nodes-overview.state';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { NodesOverviewSetActiveNode } from '@nodes/overview/nodes-overview.actions';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { filter } from 'rxjs';

@Component({
  selector: 'mina-nodes-overview-side-panel',
  templateUrl: './nodes-overview-side-panel.component.html',
  styleUrls: ['./nodes-overview-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NodesOverviewSidePanelComponent extends StoreDispatcher implements OnInit {

  node: NodesOverviewNode;

  @ViewChild('bestTipRef') private bestTipRef: ElementRef<HTMLSpanElement>;

  private interval: any;
  private secondsPassed: number = 0;
  private timeReference: number = 0;
  private timerRemoved: boolean;

  constructor(private router: Router,
              private zone: NgZone) { super(); }

  ngOnInit(): void {
    this.createTimer();
    this.listenToActiveNodesOverviewNode();
  }

  private listenToActiveNodesOverviewNode(): void {
    this.select(selectNodesOverviewActiveNode, (node: NodesOverviewNode) => {
      this.node = node;
      if (node.bestTipReceivedTimestamp) {
        if (this.timerRemoved) {
          this.createTimer();
          this.timerRemoved = false;
        }
        this.timeReference = node.bestTipReceivedTimestamp;
        this.secondsPassed = (Date.now() - this.timeReference) / 1000;
        this.updateTimeInView();
      } else {
        this.clearTimer();
        this.timerRemoved = true;
        this.bestTipRef.nativeElement.innerText = '-';
      }
      this.detect();
    }, filter(node => !!node));
  }

  closeSidePanel(): void {
    this.dispatch(NodesOverviewSetActiveNode, undefined);
    this.router.navigate([Routes.NODES, Routes.OVERVIEW], { queryParamsHandling: 'merge' });
  }

  private createTimer(): void {
    this.zone.run(() => {
      this.clearTimer();
      this.interval = setInterval(() => this.updateTimeInView(), 1000);
    });
  }

  private clearTimer(): void {
    clearInterval(this.interval);
  }

  private updateTimeInView(): void {
    const next = this.secondsPassed + 1;
    this.secondsPassed++;
    this.bestTipRef.nativeElement.innerText = NodesOverviewSidePanelComponent.getFormattedTimeToDisplay(next);
  }

  private static getFormattedTimeToDisplay(next: number): string {
    const twoDigit = (val: number) => val < 10 ? `0${val}` : val;
    let time = '';
    if (next <= 3599) {
      const min = Math.floor(next / 60);
      const sec = Math.floor(next % 60);
      time += twoDigit(min) + 'm ' + twoDigit(sec) + 's';
    } else if (next <= 86399) {
      const hour = Math.floor(next / 3600);
      const min = Math.floor(next / 60 % 60);
      time += twoDigit(hour) + 'h ' + twoDigit(min) + 'm';
    } else {
      const day = Math.floor(next / 86400);
      const hour = Math.floor(next / 3600 % 24);
      time += twoDigit(day) + 'd ' + twoDigit(hour) + 'h';
    }
    return time + ' ago';
  }
}
