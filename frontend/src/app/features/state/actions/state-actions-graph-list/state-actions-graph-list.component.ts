import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectStateActionsGroups, selectStateActionsOpenSidePanel } from '@state/actions/state-actions.state';
import { StateActionGroup } from '@shared/types/state/actions/state-action-group.type';
import { StateActionsToggleSidePanel } from '@state/actions/state-actions.actions';
import { StateActionGroupAction } from '@shared/types/state/actions/state-action-group-action.type';
import { delay } from 'rxjs';


@Component({
  selector: 'mina-state-actions-graph-list',
  templateUrl: './state-actions-graph-list.component.html',
  styleUrls: ['./state-actions-graph-list.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'p-relative' },
})
export class StateActionsGraphListComponent extends StoreDispatcher implements OnInit {

  readonly X_STEPS: string[] = ['0μs', '1μs', '10μs', '50μs', '100μs', '500μs', '1ms', '5ms', '50ms'];
  readonly RANGES: string[] = ['0μs - 1μs', '1μs - 10μs', '10μs - 50μs', '50μs - 100μs', '100μs - 500μs', '500μs - 1ms', '1ms - 5ms', '5ms - 50ms', '> 50ms'];
  readonly trackGroup = (index: number, group: StateActionGroup): string => group.groupName + group.totalTime + group.meanTime + group.count;
  readonly trackAction = (index: number, action: StateActionGroupAction): string => action.title;

  groups: StateActionGroup[] = [];
  sidePanelOpen: boolean;

  ngOnInit(): void {
    this.listenToGroups();
    this.listenToSidePanel();
  }

  private listenToSidePanel(): void {
    this.select(selectStateActionsOpenSidePanel, (open: boolean) => {
      this.sidePanelOpen = open;
      this.detect();
    }, delay(250));
  }

  private listenToGroups(): void {
    this.select(selectStateActionsGroups, (groups: StateActionGroup[]) => {
      this.groups = groups;
      this.detect();
    });
  }

  toggleSidePanel(): void {
    this.dispatch(StateActionsToggleSidePanel);
  }
}
