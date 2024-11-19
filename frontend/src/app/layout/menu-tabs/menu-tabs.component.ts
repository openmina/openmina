import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { getMergedRoute, HorizontalMenuComponent, ManualDetection, MergedRoute, removeParamsFromURL } from '@openmina/shared';
import { getAvailableFeatures } from '@shared/constants/config';
import { MENU_ITEMS, MenuItem } from '@app/layout/menu/menu.component';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { filter, map, tap } from 'rxjs';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { AppSelectors } from '@app/app.state';
import { RouterLink } from '@angular/router';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@UntilDestroy()
@Component({
  selector: 'mina-menu-tabs',
  standalone: true,
  imports: [
    HorizontalMenuComponent,
    RouterLink,
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
  readonly trackMenus = (_: number, item: MenuItem): string => item.name;

  ngOnInit(): void {
    this.listenToActiveNodeChange();
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
}
