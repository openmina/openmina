import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { any, getMergedRoute, MAX_WIDTH_700, MergedRoute } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';
import { AppSelectors } from '@app/app.state';
import { AppActions } from '@app/app.actions';
import { filter, map, Observable, Subscription, take, timer } from 'rxjs';
import { CONFIG, getFirstFeature } from '@shared/constants/config';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { Router } from '@angular/router';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'd-block h-100 w-100' },
})
export class AppComponent extends StoreDispatcher implements OnInit {

  protected readonly menu$: Observable<AppMenu> = this.select$(AppSelectors.menu);
  protected readonly showLandingPage$: Observable<boolean> = this.select$(getMergedRoute).pipe(filter(Boolean), map((route: MergedRoute) => route.url === '/'));
  subMenusLength: number = 0;
  hideToolbar: boolean = CONFIG.hideToolbar;

  private nodeUpdateSubscription: Subscription | null = null;

  constructor(private breakpointObserver: BreakpointObserver,
              private router: Router) {
    super();
    if (any(window).Cypress) {
      any(window).config = CONFIG;
      any(window).store = this.store;
    }
  }

  ngOnInit(): void {
    this.select(
      getMergedRoute,
      () => this.initAppFunctionalities(),
      filter(Boolean),
      take(1),
      filter((route: MergedRoute) => route.url !== '/'),
    );
  }

  goToWebNode(): void {
    this.router.navigate([getFirstFeature()]);
    this.initAppFunctionalities();
  }

  private initAppFunctionalities(): void {
    this.dispatch2(AppActions.init());
    if (!this.hideToolbar && !CONFIG.hideNodeStats) {
      this.scheduleNodeUpdates();
    }
    this.listenToWindowResizing();
  }

  clearNodeUpdateSubscription(): void {
    if (this.nodeUpdateSubscription) {
      this.nodeUpdateSubscription.unsubscribe();
      this.nodeUpdateSubscription = null;
    }
  }

  private scheduleNodeUpdates(): void {
    if (!this.nodeUpdateSubscription) {
      this.nodeUpdateSubscription = timer(1000, 5000).subscribe(() =>
        this.dispatch2(AppActions.getNodeDetails()),
      );
    }
  }

  private listenToWindowResizing(): void {
    this.breakpointObserver
      .observe(MAX_WIDTH_700)
      .subscribe((bs: BreakpointState) => {
        this.dispatch2(AppActions.toggleMobile({ isMobile: bs.matches }));
      });
  }

  toggleMenu(): void {
    this.dispatch2(AppActions.toggleMenuOpening());
  }

  onSubmenusLengthChange(length: number): void {
    this.subMenusLength = length;
  }
}
