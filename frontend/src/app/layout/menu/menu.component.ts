import { ChangeDetectionStrategy, Component, Inject, OnInit } from '@angular/core';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectActiveNode, selectAppMenu } from '@app/app.state';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import { AppMenu } from '@shared/types/app/app-menu.type';
import {
  APP_CHANGE_MENU_COLLAPSING,
  APP_TOGGLE_MENU_OPENING,
  AppChangeMenuCollapsing,
  AppToggleMenuOpening
} from '@app/app.actions';
import {
  ManualDetection,
  removeParamsFromURL,
  ThemeSwitcherService,
  ThemeType,
  TooltipPosition
} from '@openmina/shared';
import { DOCUMENT } from '@angular/common';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { filter, map, tap } from 'rxjs';
import { CONFIG, getAvailableFeatures } from '@shared/constants/config';
import { NavigationEnd, Router } from '@angular/router';

interface MenuItem {
  name: string;
  icon: string;
  tooltip?: string;
}

const MENU_ITEMS: MenuItem[] = [
  { name: 'Dashboard', icon: 'dashboard' },
  { name: 'Nodes', icon: 'margin' },
  { name: 'State', icon: 'code_blocks' },
  { name: 'SNARKs', icon: 'assignment_turned_in' },
  { name: 'Testing Tool', icon: 'build' },
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

  menuItems: MenuItem[] = this.allowedMenuItems;
  menu: AppMenu;
  currentTheme: ThemeType;
  appIdentifier: string = CONFIG.identifier;
  activeNode: MinaNode;
  activeRoute: string;

  constructor(@Inject(DOCUMENT) private readonly document: Document,
              private router: Router,
              private store: Store<MinaState>,
              private themeService: ThemeSwitcherService) { super(); }

  ngOnInit(): void {
    this.currentTheme = this.themeService.activeTheme;
    this.listenToCollapsingMenu();
    this.listenToActiveNodeChange();
    let lastUrl: string;
    this.router.events.pipe(
      filter(event => event instanceof NavigationEnd),
      map((event: any) => (event as NavigationEnd).urlAfterRedirects),
      filter(url => url !== lastUrl),
      tap(url => lastUrl = url),
      map(removeParamsFromURL),
      map(url => url.split('/')[1]),
    ).subscribe((url: string) => {
      this.activeRoute = url;
      this.detect();
    });
  }

  changeTheme(): void {
    this.themeService.changeTheme();
    this.currentTheme = this.themeService.activeTheme;
  }

  private listenToCollapsingMenu(): void {
    this.store.select(selectAppMenu)
      .pipe(untilDestroyed(this))
      .subscribe((menu: AppMenu) => {
        this.menu = menu;
        this.detect();
      });
  }

  private listenToActiveNodeChange(): void {
    this.store.select(selectActiveNode)
      .pipe(
        untilDestroyed(this),
        filter(node => !!node),
      )
      .subscribe((node: MinaNode) => {
        this.activeNode = node;
        this.menuItems = this.allowedMenuItems;
        this.detect();
      });
  }

  private get allowedMenuItems(): MenuItem[] {
    const features = getAvailableFeatures(this.activeNode || {} as any);
    return MENU_ITEMS.filter((opt: MenuItem) => features.find(f => f === opt.name.toLowerCase().split(' ').join('-')));
  }

  showHideMenu(): void {
    if (this.menu.isMobile) {
      this.store.dispatch<AppToggleMenuOpening>({ type: APP_TOGGLE_MENU_OPENING });
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
    this.store.dispatch<AppChangeMenuCollapsing>({ type: APP_CHANGE_MENU_COLLAPSING, payload: !this.menu.collapsed });
  }

  protected readonly TooltipPosition = TooltipPosition;
}
