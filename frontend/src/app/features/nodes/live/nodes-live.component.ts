import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectActiveNode } from '@app/app.state';
import { skip, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { NodesLiveClose, NodesLiveGetNodes, NodesLiveInit } from '@nodes/live/nodes-live.actions';
import { selectNodesLiveOpenSidePanel } from '@nodes/live/nodes-live.state';

@Component({
  selector: 'mina-nodes-live',
  templateUrl: './nodes-live.component.html',
  styleUrls: ['./nodes-live.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NodesLiveComponent extends StoreDispatcher implements OnInit, OnDestroy {

  openSidePanel: boolean;

  constructor(public el: ElementRef<HTMLElement>) { super(); }

  ngOnInit(): void {
    this.listenToNodeChange();
    this.listenToSidePanelOpening();
  }

  private listenToNodeChange(): void {
    this.select(selectActiveNode, () => {
      this.dispatch(NodesLiveInit);
      this.dispatch(NodesLiveGetNodes, { force: true });
    }, skip(1));

    timer(0, 2000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        this.dispatch(NodesLiveGetNodes);
      });
  }

  private listenToSidePanelOpening(): void {
    this.select(selectNodesLiveOpenSidePanel, (open: boolean) => {
      this.openSidePanel = !!open;
      this.detect();
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(NodesLiveClose);
  }
}
