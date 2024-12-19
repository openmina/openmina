import { ChangeDetectionStrategy, Component, OnInit, TemplateRef, ViewChild, ViewContainerRef } from '@angular/core';
import {
  getMergedRoute,
  HorizontalMenuComponent, isDesktop,
  MergedRoute,
  OpenminaEagerSharedModule,
  removeParamsFromURL,
  ThemeSwitcherService,
  ThemeType,
} from '@openmina/shared';
import { getAvailableFeatures } from '@shared/constants/config';
import { MENU_ITEMS, MenuItem } from '@app/layout/menu/menu.component';
import { filter, map, merge, skip, take, tap } from 'rxjs';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { AppSelectors } from '@app/app.state';
import { RouterLink } from '@angular/router';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppEnvBuild } from '@shared/types/app/app-env-build.type';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { ComponentPortal, TemplatePortal } from '@angular/cdk/portal';
import { MinaNetwork } from '@shared/types/core/mina/mina.type';
import { EnvBuildModalComponent } from '@app/layout/env-build-modal/env-build-modal.component';

@UntilDestroy()
@Component({
  selector: 'mina-menu-tabs',
  standalone: true,
  imports: [
    HorizontalMenuComponent,
    RouterLink,
    OpenminaEagerSharedModule,
  ],
  templateUrl: './menu-tabs.component.html',
  styleUrl: './menu-tabs.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column w-100' },
})
export class MenuTabsComponent extends StoreDispatcher implements OnInit {

  menuItems: MenuItem[] = this.allowedMenuItems;
  activeRoute: string;
  activeNode: MinaNode;
  network?: MinaNetwork;
  chainId?: string;
  envBuild: AppEnvBuild;
  isOpenMore: boolean;
  currentTheme: ThemeType;
  readonly trackMenus = (_: number, item: MenuItem): string => item.name;
  readonly ThemeType = ThemeType;

  @ViewChild('dropdown') private dropdown: TemplateRef<void>;

  private overlayRef: OverlayRef;

  constructor(private overlay: Overlay,
              private viewContainerRef: ViewContainerRef,
              private themeService: ThemeSwitcherService) {super();}

  ngOnInit(): void {
    this.currentTheme = this.themeService.activeTheme;
    this.listenToActiveNodeChange();
    this.listenToEnvBuild();
    this.listenToNetwork();

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

  changeTheme(type: ThemeType): void {
    this.closeOverlay();
    if (type === this.currentTheme) {
      return;
    }
    this.themeService.changeTheme();
    this.currentTheme = this.themeService.activeTheme;
  }


  private listenToActiveNodeChange(): void {
    this.select(AppSelectors.activeNode, (node: MinaNode) => {
      this.activeNode = node;
      this.menuItems = this.allowedMenuItems;
      this.detect();
    }, filter(node => !!node));
  }

  private get allowedMenuItems(): MenuItem[] {
    const features = getAvailableFeatures(this.activeNode || { features: {} } as any);
    return MENU_ITEMS.filter((opt: MenuItem) => features.find(f => f === opt.name.toLowerCase().split(' ').join('-')));
  }

  private listenToEnvBuild(): void {
    this.select(AppSelectors.envBuild, (env: AppEnvBuild) => {
      this.envBuild = env;
      this.detect();
    });
  }

  private listenToNetwork(): void {
    this.select(AppSelectors.activeNodeDetails, ({ chainId, network }) => {
      this.chainId = chainId;
      this.network = network as MinaNetwork;
      this.detect();
    }, filter(Boolean));
  }

  openMore(anchor: HTMLDivElement): void {
    this.isOpenMore = true;
    if (this.overlayRef?.hasAttached()) {
      this.closeOverlay();
      return;
    }
    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      width: window.innerWidth - 6,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(anchor)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'bottom',
          offsetY: -10,
          offsetX: -4,
        }]),
    });

    const portal = new TemplatePortal(this.dropdown, this.viewContainerRef);
    this.overlayRef.attach(portal);
  }

  closeOverlay(): void {
    this.overlayRef?.dispose();
    this.isOpenMore = false;
    this.detect();
  }

  openEnvBuildModal(): void {
    this.closeOverlay();
    this.overlayRef = this.overlay.create({
      hasBackdrop: true,
      backdropClass: 'openmina-backdrop',
      width: 'calc(100% - 8px)',
      height: 'calc(100% - 8px)',
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
