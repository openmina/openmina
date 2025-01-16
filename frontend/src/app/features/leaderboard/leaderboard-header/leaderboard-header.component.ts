import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { getMergedRoute, isDesktop, MergedRoute } from '@openmina/shared';
import { animate, state, style, transition, trigger } from '@angular/animations';

@Component({
  selector: 'mina-leaderboard-header',
  templateUrl: './leaderboard-header.component.html',
  styleUrl: './leaderboard-header.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column' },
  animations: [
    trigger('dropdownAnimation', [
      state('closed', style({
        height: '0',
        opacity: '0',
        overflow: 'hidden',
      })),
      state('open', style({
        height: '*',
        opacity: '1',
      })),
      transition('closed => open', [
        animate('300ms ease-out'),
      ]),
      transition('open => closed', [
        animate('200ms ease-in'),
      ]),
    ]),
  ],
})
export class LeaderboardHeaderComponent extends StoreDispatcher implements OnInit {

  route: string;
  isMenuOpen: boolean = isDesktop();

  ngOnInit(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      this.route = route.url;
      this.detect();
    });
  }

  closeMenu(): void {
    if (isDesktop()) {
      return;
    }
    this.isMenuOpen = false;
  }

  toggleMenu(): void {
    if (isDesktop()) {
      return;
    }
    this.isMenuOpen = !this.isMenuOpen;
  }
}
