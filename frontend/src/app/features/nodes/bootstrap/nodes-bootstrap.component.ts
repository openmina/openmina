import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NodesBootstrapClose, NodesBootstrapGetNodes, NodesBootstrapInit } from '@nodes/bootstrap/nodes-bootstrap.actions';
import { selectNodesBootstrapOpenSidePanel } from '@nodes/bootstrap/nodes-bootstrap.state';
import { skip, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { selectActiveNode } from '@app/app.state';

@Component({
  selector: 'mina-nodes-bootstrap',
  templateUrl: './nodes-bootstrap.component.html',
  styleUrls: ['./nodes-bootstrap.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NodesBootstrapComponent extends StoreDispatcher implements OnInit, OnDestroy {

  openSidePanel: boolean;

  constructor(public el: ElementRef<HTMLElement>) { super(); }

  ngOnInit(): void {
    this.listenToNodeChange();
    this.listenToSidePanelOpening();
  }

  private listenToNodeChange(): void {
    this.select(selectActiveNode, () => {
      this.dispatch(NodesBootstrapInit);
      this.dispatch(NodesBootstrapGetNodes, { force: true });
    }, skip(1));

    timer(0, 10000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        this.dispatch(NodesBootstrapGetNodes);
      });
  }

  private listenToSidePanelOpening(): void {
    this.select(selectNodesBootstrapOpenSidePanel, (open: boolean) => {
      this.openSidePanel = !!open;
      this.detect();
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(NodesBootstrapClose);
  }
}
