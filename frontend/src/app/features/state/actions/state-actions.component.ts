import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { StateActionsClose, StateActionsGetEarliestSlot } from '@state/actions/state-actions.actions';
import { selectStateActionsOpenSidePanel } from '@state/actions/state-actions.state';
import { Subscription, timer } from 'rxjs';
import { selectActiveNode } from '@app/app.state';
import { untilDestroyed } from '@ngneat/until-destroy';

@Component({
  selector: 'mina-state-actions',
  templateUrl: './state-actions.component.html',
  styleUrls: ['./state-actions.component.scss'],
  host: { class: 'flex-column h-100' },
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StateActionsComponent extends StoreDispatcher implements OnInit, OnDestroy {

  show: boolean;

  constructor(public el: ElementRef) { super(); }

  ngOnInit(): void {
    this.checkEarliestSlot();
    this.listenToSidePanelChange();
  }

  private listenToSidePanelChange(): void {
    this.select(selectStateActionsOpenSidePanel, (open: boolean) => {
      this.show = open;
      this.detect();
    });
  }

  private checkEarliestSlot(): void {
    let subscription: Subscription;

    this.select(selectActiveNode, () => {
      subscription?.unsubscribe();
      subscription = timer(0, 20000)
        .pipe(untilDestroyed(this))
        .subscribe(() => {
          this.dispatch(StateActionsGetEarliestSlot);
        });
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(StateActionsClose);
  }
}
