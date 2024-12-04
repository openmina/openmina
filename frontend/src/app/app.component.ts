import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { any, getMergedRoute, getWindow, isBrowser, isDesktop, MAX_WIDTH_700, MergedRoute, safelyExecuteInBrowser } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';
import { AppSelectors } from '@app/app.state';
import { AppActions } from '@app/app.actions';
import { filter, map, Observable, Subscription, take, timer } from 'rxjs';
import { CONFIG } from '@shared/constants/config';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { WebNodeService } from '@core/services/web-node.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'd-block h-100 w-100' },
})
export class AppComponent extends StoreDispatcher implements OnInit {

  readonly menu$: Observable<AppMenu> = this.select$(AppSelectors.menu);
  readonly showLandingPage$: Observable<boolean> = this.select$(getMergedRoute).pipe(filter(Boolean), map((route: MergedRoute) => route.url === '/' || route.url.startsWith('/?')));
  readonly showLoadingWebNodePage$: Observable<boolean> = this.select$(getMergedRoute).pipe(filter(Boolean), map((route: MergedRoute) => route.url.startsWith(`/${Routes.LOADING_WEB_NODE}`)));
  subMenusLength: number = 0;
  hideToolbar: boolean = CONFIG.hideToolbar;
  loaded: boolean;
  isDesktop: boolean = isDesktop();

  private nodeUpdateSubscription: Subscription | null = null;

  constructor(private breakpointObserver: BreakpointObserver,
              private router: Router,
              private webNodeService: WebNodeService) {
    super();
    safelyExecuteInBrowser(() => {
      if (any(window).Cypress) {
        any(window).config = CONFIG;
        any(window).store = this.store;
      }
    });
  }

  ngOnInit(): void {
    if (isBrowser()) {
      const args = new URLSearchParams(window.location.search).get('a');
      if (!!args) {
        localStorage.setItem('webnodeArgs', args);
      }
    }

    this.select(
      getMergedRoute,
      () => this.initAppFunctionalities(),
      filter(Boolean),
      take(1),
      filter((route: MergedRoute) => route.url !== '/' && !route.url.startsWith('/?')),
    );
    this.select(
      getMergedRoute,
      () => {
        this.loaded = true;
        this.detect();
      },
      filter(Boolean),
      take(1),
    );
  }

  goToWebNode(): void {
    this.router.navigate([Routes.LOADING_WEB_NODE], { queryParamsHandling: 'merge' });
    this.initAppFunctionalities();
  }

  private initAppFunctionalities(): void {
    if (this.webNodeService.hasWebNodeConfig() && !this.webNodeService.isWebNodeLoaded()) {
      if (!getWindow()?.location.href.includes(`/${Routes.LOADING_WEB_NODE}`)) {
        this.router.navigate([Routes.LOADING_WEB_NODE], { queryParamsHandling: 'preserve' });
      }
    }
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
