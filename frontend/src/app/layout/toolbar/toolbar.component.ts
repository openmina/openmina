import { ChangeDetectionStrategy, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { filter, map } from 'rxjs';
import { AppSelectors } from '@app/app.state';
import { getMergedRoute, MergedRoute, removeParamsFromURL, TooltipService } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { AppActions } from '@app/app.actions';
import { selectLoadingStateLength } from '@app/layout/toolbar/loading.reducer';
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

  @ViewChild('loadingRef') private loadingRef: ElementRef<HTMLDivElement>;

  constructor(private tooltipService: TooltipService) { super(); }

  ngOnInit(): void {
    this.listenToRouterChange();
    this.listenToMenuChange();
    this.listenToLoading();
  }

  private listenToLoading(): void {
    const displayNone: string = 'd-none';
    const classList = this.loadingRef.nativeElement.classList;

    this.select(selectLoadingStateLength, (length: number) => {
      if (length > 0) {
        classList.remove(displayNone);
      } else {
        classList.add(displayNone);
      }
    });
  }

  private listenToMenuChange(): void {
    this.select(AppSelectors.menu, (menu: AppMenu) => {
      this.isMobile = menu.isMobile;
      this.detect();
    }, filter(menu => menu.isMobile !== this.isMobile));
  }

  toggleTooltips(): void {
    this.tooltipService.toggleTooltips();
  }

  toggleMenu(): void {
    this.dispatch2(AppActions.toggleMenuOpening());
  }

  private listenToRouterChange(): void {
    this.select(getMergedRoute, (url: string) => {
      this.title = removeParamsFromURL(url).split('/')[1].replace(/-/g, ' ');
      this.detect();
    }, filter(Boolean), map((route: MergedRoute) => route.url));
  }
}
