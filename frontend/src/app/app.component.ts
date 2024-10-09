import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { any, ManualDetection, MAX_WIDTH_700 } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';
import { AppSelectors } from '@app/app.state';
import { AppActions } from '@app/app.actions';
import { Observable, timer } from 'rxjs';
import { CONFIG } from '@shared/constants/config';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'd-block h-100 w-100' },
})
export class AppComponent extends ManualDetection implements OnInit {

  protected readonly menu$: Observable<AppMenu> = this.store.select(AppSelectors.menu);
  subMenusLength: number = 0;
  hideToolbar: boolean = CONFIG.hideToolbar;

  constructor(private store: Store<MinaState>,
              private breakpointObserver: BreakpointObserver) {
    super();
    if (any(window).Cypress) {
      any(window).config = CONFIG;
      any(window).store = store;
    }
  }

  ngOnInit(): void {
    if (!this.hideToolbar && !CONFIG.hideNodeStats) {
      this.scheduleNodeUpdates();
    }
    this.listenToWindowResizing();
  }

  private scheduleNodeUpdates(): void {
    timer(1000, 5000).subscribe(() => this.store.dispatch(AppActions.getNodeDetails()));
  }

  private listenToWindowResizing(): void {
    this.breakpointObserver
      .observe(MAX_WIDTH_700)
      .subscribe((bs: BreakpointState) => {
        this.store.dispatch(AppActions.toggleMobile({ isMobile: bs.matches }));
      });
  }

  toggleMenu(): void {
    this.store.dispatch(AppActions.toggleMenuOpening());
  }

  onSubmenusLengthChange(length: number): void {
    this.subMenusLength = length;
  }
}
