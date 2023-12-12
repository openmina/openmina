import { ChangeDetectionStrategy, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { ActivatedRoute, NavigationEnd, Router } from '@angular/router';
import { filter, map } from 'rxjs';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectAppMenu } from '@app/app.state';
import { getMergedRoute, ManualDetection, TooltipService } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { APP_TOGGLE_MENU_OPENING, AppToggleMenuOpening } from '@app/app.actions';
import { selectLoadingStateLength } from '@app/layout/toolbar/loading.reducer';
import { Routes } from '@shared/enums/routes.enum';

@Component({
  selector: 'mina-toolbar',
  templateUrl: './toolbar.component.html',
  styleUrls: ['./toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row align-center border-bottom' },
})
export class ToolbarComponent extends ManualDetection implements OnInit {

  title: string = 'Loading';
  isMobile: boolean;
  switchForbidden: boolean;
  hideSwitcher: boolean;

  @ViewChild('loadingRef') private loadingRef: ElementRef<HTMLDivElement>;

  constructor(private router: Router,
              private store: Store<MinaState>,
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
    this.store.select(selectAppMenu)
      .pipe(filter(menu => menu.isMobile !== this.isMobile))
      .subscribe((menu: AppMenu) => {
        this.isMobile = menu.isMobile;
        this.detect();
      });
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
          this.buildSwitchForbidden();
          this.detect();
        }
      });
  }

  toggleTooltips(): void {
    this.tooltipService.toggleTooltips();
  }

  toggleMenu(): void {
    this.store.dispatch<AppToggleMenuOpening>({ type: APP_TOGGLE_MENU_OPENING });
  }

  private listenToRouterChange(): void {
    this.store.select(getMergedRoute)
      .subscribe(() => {
        this.buildSwitchForbidden();
        this.detect();
      });
  }

  private buildSwitchForbidden(): void {
    this.switchForbidden = location.pathname.includes(Routes.NODES + '/' + Routes.OVERVIEW);
    this.hideSwitcher = location.pathname.includes(Routes.TESTING_TOOL);
  }
}
