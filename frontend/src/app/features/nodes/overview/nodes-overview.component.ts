import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectNodesOverviewActiveNode } from '@nodes/overview/nodes-overview.state';
import { NodesOverviewClose, NodesOverviewGetNodes } from '@nodes/overview/nodes-overview.actions';
import { timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';

@Component({
  selector: 'mina-nodes-overview',
  templateUrl: './nodes-overview.component.html',
  styleUrls: ['./nodes-overview.component.scss'],
  host: { class: 'flex-column h-100' },
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NodesOverviewComponent extends StoreDispatcher implements OnInit, OnDestroy {

  isActiveRow: boolean;

  constructor(public el: ElementRef) { super(); }

  ngOnInit(): void {
    timer(0, 5000).pipe(
      untilDestroyed(this),
    ).subscribe(() => {
      this.dispatch(NodesOverviewGetNodes);
    });
    this.listenToSidePanelChange();
  }

  private listenToSidePanelChange(): void {
    this.select(selectNodesOverviewActiveNode, node => {
      if (node && !this.isActiveRow) {
        this.isActiveRow = true;
        this.detect();
      } else if (!node && this.isActiveRow) {
        this.isActiveRow = false;
        this.detect();
      }
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(NodesOverviewClose);
  }
}
