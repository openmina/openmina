import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { ManualDetection, MAX_WIDTH_700 } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';
import { selectAppMenu } from '@app/app.state';
import {
  APP_TOGGLE_MENU_OPENING,
  APP_TOGGLE_MOBILE,
  AppToggleMenuOpening,
  AppToggleMobile
} from '@app/app.actions';
import { Observable } from 'rxjs';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'd-block h-100 w-100' },
})
export class AppComponent extends ManualDetection implements OnInit {

  menu$: Observable<AppMenu> = this.store.select(selectAppMenu);

  constructor(private store: Store<MinaState>,
              private breakpointObserver: BreakpointObserver) {
    super();
    if ((window as any).Cypress) {
      (window as any).store = store;
    }
  }

  ngOnInit(): void {
    this.listenToWindowResizing();
  }

  private listenToWindowResizing(): void {
    this.breakpointObserver
      .observe(MAX_WIDTH_700)
      .subscribe((bs: BreakpointState) => {
        this.store.dispatch<AppToggleMobile>({ type: APP_TOGGLE_MOBILE, payload: { isMobile: bs.matches } });
      });
  }

  toggleMenu(): void {
    this.store.dispatch<AppToggleMenuOpening>({ type: APP_TOGGLE_MENU_OPENING });
  }
}
