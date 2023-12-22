import { ChangeDetectionStrategy, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { ActivatedRoute, NavigationEnd, Router } from '@angular/router';
import { filter, map } from 'rxjs';
import { selectAppMenu } from '@app/app.state';
import { getMergedRoute, TooltipService } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { AppToggleMenuOpening } from '@app/app.actions';
import { selectLoadingStateLength } from '@app/layout/toolbar/loading.reducer';
import { Routes } from '@shared/enums/routes.enum';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-toolbar',
  templateUrl: './toolbar.component.html',
  styleUrls: ['./toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row align-center border-bottom' },
})
export class ToolbarComponent extends StoreDispatcher implements OnInit {

  title: string = 'Loading';
  isMobile: boolean;
  switchForbidden: boolean;
  hideSwitcher: boolean;

  @ViewChild('loadingRef') private loadingRef: ElementRef<HTMLDivElement>;

  constructor(private router: Router,
              private tooltipService: TooltipService) { super(); }

  ngOnInit(): void {
    this.listenToTitleChange();
    this.listenToMenuChange();
    this.listenToLoading();
    this.listenToRouterChange();
  }

  private listenToLoading(): void {
    const displayNone: string = 'd-none';
    const classList = this.loadingRef.nativeElement.classList;

    this.store.select(selectLoadingStateLength)
      .subscribe((length: number) => {
        if (length > 0) {
          classList.remove(displayNone);
        } else {
          classList.add(displayNone);
        }
      });
  }

  private listenToMenuChange(): void {
    this.select(selectAppMenu, (menu: AppMenu) => {
      this.isMobile = menu.isMobile;
      this.detect();
    }, filter(menu => menu.isMobile !== this.isMobile));
  }

  private listenToTitleChange(): void {
    this.router.events
      .pipe(
        filter((event) => event instanceof NavigationEnd),
        map(() => {
          let route: ActivatedRoute = this.router.routerState.root;
          while (route!.firstChild) {
            route = route.firstChild;
          }
          return route.snapshot.data[Object.getOwnPropertySymbols(route.snapshot.data)[0]];
        }),
      )
      .subscribe((title: string) => {
        if (title) {
          this.title = title.split('- ')[1];
          this.nodeSwitchForbidden();
          this.detect();
        }
      });
  }

  toggleTooltips(): void {
    this.tooltipService.toggleTooltips();
  }

  toggleMenu(): void {
    this.dispatch(AppToggleMenuOpening);
  }

  private listenToRouterChange(): void {
    this.select(getMergedRoute, () => {
      this.nodeSwitchForbidden();
      this.detect();
    });
  }

  private nodeSwitchForbidden(): void {
    this.switchForbidden = location.pathname.includes(Routes.NODES + '/' + Routes.OVERVIEW);
    this.hideSwitcher = location.pathname.includes(Routes.TESTING_TOOL);
  }
}
