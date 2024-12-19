import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { AppSelectors } from '@app/app.state';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { AppActions } from '@app/app.actions';
import {
  getMergedRoute, isDesktop,
  isMobile,
  ManualDetection,
  MergedRoute,
  removeParamsFromURL,
  ThemeSwitcherService,
  ThemeType,
  TooltipPosition,
} from '@openmina/shared';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { filter, map, merge, take, tap } from 'rxjs';
import { CONFIG, getAvailableFeatures } from '@shared/constants/config';
import { MinaNetwork } from '@shared/types/core/mina/mina.type';
import { AppEnvBuild } from '@shared/types/app/app-env-build.type';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { ComponentPortal } from '@angular/cdk/portal';
import { EnvBuildModalComponent } from '@app/layout/env-build-modal/env-build-modal.component';

export interface MenuItem {
  name: string;
  icon: string;
  tooltip?: string;
}

export const MENU_ITEMS: MenuItem[] = [
  { name: 'Dashboard', icon: 'dashboard' },
  { name: 'Block Production', icon: 'library_add' },
  { name: 'Nodes', icon: 'margin' },
  { name: 'Resources', icon: 'analytics' },
  { name: 'Mempool', icon: 'blur_circular' },
  { name: 'Network', icon: 'account_tree' },
  { name: 'State', icon: 'code_blocks' },
  { name: 'SNARKs', icon: 'assignment_turned_in' },
  { name: 'Benchmarks', icon: 'dynamic_form' },
  { name: 'Fuzzing', icon: 'shuffle' },
];

@UntilDestroy()
@Component({
  selector: 'mina-menu',
  templateUrl: './menu.component.html',
  styleUrls: ['./menu.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column flex-between h-100 pb-5' },
})
export class MenuComponent extends ManualDetection implements OnInit {

  protected readonly TooltipPosition = TooltipPosition;

  menuItems: MenuItem[] = this.allowedMenuItems;
  menu: AppMenu;
  currentTheme: ThemeType;
  appIdentifier: string = CONFIG.identifier;
  hideNodeStats: boolean = CONFIG.hideNodeStats;
  activeNode: MinaNode;
  activeRoute: string;
  network?: MinaNetwork;
  chainId?: string;
  envBuild: AppEnvBuild;

  private overlayRef: OverlayRef;

  constructor(private overlay: Overlay,
              private store: Store<MinaState>,
              private themeService: ThemeSwitcherService) { super(); }

  ngOnInit(): void {
    this.currentTheme = this.themeService.activeTheme;
    this.listenToCollapsingMenu();
    this.listenToActiveNodeChange();
    this.listenToEnvBuild();

    let lastUrl: string;
    this.store.select(getMergedRoute)
      .pipe(
        filter(Boolean),
        map((route: MergedRoute) => route.url),
        filter(url => url !== lastUrl),
        tap(url => lastUrl = url),
        untilDestroyed(this),
      )
      .subscribe((url: string) => {
        this.activeRoute = removeParamsFromURL(url).split('/')[1];
        this.detect();
      });
  }

  changeTheme(): void {
    this.themeService.changeTheme();
    this.currentTheme = this.themeService.activeTheme;
  }

  private listenToCollapsingMenu(): void {
    this.store.select(AppSelectors.menu)
      .pipe(untilDestroyed(this))
      .subscribe((menu: AppMenu) => {
        this.menu = menu;
        this.detect();
      });
  }

  private listenToActiveNodeChange(): void {
    this.store.select(AppSelectors.activeNode)
      .pipe(
        filter(node => !!node),
        untilDestroyed(this),
      )
      .subscribe((node: MinaNode) => {
        this.activeNode = node;
        this.menuItems = this.allowedMenuItems;
        this.detect();
      });

    this.store.select(AppSelectors.activeNodeDetails)
      .pipe(
        filter(Boolean),
        untilDestroyed(this),
      )
      .subscribe(({ chainId, network }) => {
        this.chainId = chainId;
        this.network = network as MinaNetwork;
        this.detect();
      });
  }

  private listenToEnvBuild(): void {
    this.store.select(AppSelectors.envBuild)
      .pipe(
        filter(Boolean),
        untilDestroyed(this),
      )
      .subscribe((env: AppEnvBuild | undefined) => {
        this.envBuild = env;
        this.detect();
      });
  }

  private get allowedMenuItems(): MenuItem[] {
    const features = getAvailableFeatures(this.activeNode || { features: {} } as any);
    return MENU_ITEMS.filter((opt: MenuItem) => features.find(f => f === opt.name.toLowerCase().split(' ').join('-')));
  }

  showHideMenu(): void {
    if (this.menu.isMobile) {
      this.store.dispatch(AppActions.toggleMenuOpening());
    }
  }

  toggleMenu(): void {
    if (this.menu.isMobile) {
      this.showHideMenu();
    } else {
      this.collapseMenu();
    }
  }

  collapseMenu(): void {
    this.store.dispatch(AppActions.changeMenuCollapsing({ isCollapsing: !this.menu.collapsed }));
  }

  openEnvBuildModal(): void {
    this.overlayRef = this.overlay.create({
      hasBackdrop: true,
      backdropClass: 'openmina-backdrop',
      width: '99%',
      height: '99%',
      maxWidth: 600,
      maxHeight: 460,
      positionStrategy: this.overlay.position().global().centerVertically().centerHorizontally(),
    });

    const portal = new ComponentPortal(EnvBuildModalComponent);
    const component = this.overlayRef.attach<EnvBuildModalComponent>(portal);
    component.instance.envBuild = this.envBuild;
    component.instance.detect();

    merge(
      component.instance.close,
      this.overlayRef.backdropClick(),
    )
      .pipe(take(1))
      .subscribe(() => this.overlayRef.dispose());
  }
}
