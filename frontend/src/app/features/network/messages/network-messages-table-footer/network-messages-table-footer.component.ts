import { ChangeDetectionStrategy, Component, ComponentRef, EventEmitter, OnInit, Output } from '@angular/core';
import { NetworkMessagesState, selectNetworkTimestampInterval } from '@network/messages/network-messages.state';
import {
  NetworkMessagesGetPaginatedMessages,
  NetworkMessagesGoLive,
  NetworkMessagesPause,
  NetworkMessagesSetTimestampInterval,
} from '@network/messages/network-messages.actions';
import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { ComponentPortal } from '@angular/cdk/portal';
import { filter, take } from 'rxjs';
import {
  getMergedRoute,
  IntervalSelectComponent,
  MergedRoute,
  ONE_THOUSAND,
  TimestampInterval
} from '@openmina/shared';
import { DatePipe } from '@angular/common';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { selectNetworkMessagesState } from '@network/network.state';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-network-messages-table-footer',
  templateUrl: './network-messages-table-footer.component.html',
  styleUrls: ['./network-messages-table-footer.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'fx-row-vert-cent border-top h-lg' },
})
export class NetworkMessagesTableFooterComponent extends StoreDispatcher implements OnInit {

  @Output() onScrollTopClick: EventEmitter<void> = new EventEmitter<void>();

  state: NetworkMessagesState;
  isFirstPage: boolean;
  isLastPage: boolean;
  activeInterval: string;

  private overlayRef: OverlayRef;
  private intervalSelectComponent: ComponentRef<IntervalSelectComponent>;
  private currentTimestamp: TimestampInterval;
  private urlMessageId: string;

  constructor(private datePipe: DatePipe,
              private overlay: Overlay,
              private router: Router) { super(); }

  ngOnInit(): void {
    this.listenToNetworkMessages();
    this.listenToTimestampIntervalChange();
    this.listenToRouteChange();
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      this.urlMessageId = route.params['messageId'];
    });
  }

  private listenToNetworkMessages(): void {
    this.select(selectNetworkMessagesState, (state: NetworkMessagesState) => {
      this.state = state;
      this.isFirstPage = state.activePage.start?.id === 0
        || !state.activePage.firstPageIdWithTimestamp && state.messages.length < state.limit
        || state.activePage.firstPageIdWithFilters === state.activePage.start?.id
        || state.activePage.firstPageIdWithTimestamp && state.activePage.start?.id <= state.activePage.firstPageIdWithTimestamp;
      this.isLastPage = state.stream
        || (
          !state.activePage.firstPageIdWithTimestamp
          && (state.pages.length === 0 || state.activePage.end?.id === state.pages[state.pages.length - 1])
        )
        || state.activePage.lastPageIdWithFilters === state.activePage.end.id
        || state.activePage.lastPageIdWithTimestamp === state.activePage.end.id;
      this.detect();
    });
  }

  private listenToTimestampIntervalChange(): void {
    this.select(selectNetworkTimestampInterval, (timestamp: TimestampInterval) => {
      this.currentTimestamp = timestamp;
      const from = this.datePipe.transform(timestamp.from * ONE_THOUSAND, 'MMM d, H:mm:ss');
      let to = this.datePipe.transform(timestamp.to * ONE_THOUSAND, 'MMM d, H:mm:ss');
      if (from.split(',')[0] === to.split(',')[0]) {
        to = this.datePipe.transform(timestamp.to * ONE_THOUSAND, 'H:mm:ss');
      }
      this.activeInterval = from + ' - ' + to;
      this.detect();
    }, filter(timestamp => !!timestamp.from));
  }

  goLive(): void {
    this.dispatch(NetworkMessagesGoLive);
  }

  pause(): void {
    this.dispatch(NetworkMessagesPause);
  }

  previousPage(): void {
    let payload: NetworkMessagesGetPaginatedMessages['payload'];
    if (this.currentTimestamp) {
      payload = {
        id: this.state.activePage.start.id - this.state.limit,
        timestamp: { from: undefined, to: this.currentTimestamp.to },
        direction: NetworkMessagesDirection.FORWARD,
      };
    } else {
      payload = {
        id: this.state.activePage.start.id - 1,
        direction: NetworkMessagesDirection.REVERSE,
      };
    }
    this.dispatch(NetworkMessagesGetPaginatedMessages, payload);
  }

  nextPage(): void {
    const payload: NetworkMessagesGetPaginatedMessages['payload'] = {
      id: this.state.activePage.end.id + 1,
      direction: NetworkMessagesDirection.FORWARD,
    };
    if (this.currentTimestamp) {
      payload.timestamp = { from: undefined, to: this.currentTimestamp.to };
    }
    this.dispatch(NetworkMessagesGetPaginatedMessages, payload);
  }

  firstPage(): void {
    let payload: NetworkMessagesGetPaginatedMessages['payload'];
    if (this.currentTimestamp) {
      payload = {
        id: this.state.activePage.firstPageIdWithTimestamp,
        timestamp: { from: undefined, to: this.currentTimestamp.to },
        direction: NetworkMessagesDirection.FORWARD,
      };
    } else {
      payload = {
        id: 0,
        direction: NetworkMessagesDirection.FORWARD,
      };
    }
    this.dispatch(NetworkMessagesGetPaginatedMessages, payload);
  }

  lastPage(): void {
    let payload: NetworkMessagesGetPaginatedMessages['payload'];
    if (this.currentTimestamp) {
      payload = {
        timestamp: { from: this.currentTimestamp.to, to: this.currentTimestamp.from },
        direction: NetworkMessagesDirection.REVERSE,
      };
    } else {
      payload = {
        id: this.state.pages[this.state.pages.length - 1],
        direction: NetworkMessagesDirection.REVERSE,
      };
    }
    this.dispatch(NetworkMessagesGetPaginatedMessages, payload);
  }

  openIntervalPicker(event?: MouseEvent): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
      return;
    }

    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(event?.target as HTMLElement)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: -35,
        }]),
    });
    event?.stopPropagation();

    const portal = new ComponentPortal(IntervalSelectComponent);
    this.intervalSelectComponent = this.overlayRef.attach<IntervalSelectComponent>(portal);
    this.intervalSelectComponent.instance.from = this.currentTimestamp?.from;
    this.intervalSelectComponent.instance.to = this.currentTimestamp?.to;
    setTimeout(() => {
      this.intervalSelectComponent.instance.animate = true;
      this.intervalSelectComponent.instance.detect();
    });
    this.intervalSelectComponent.instance.onConfirm
      .pipe(take(1))
      .subscribe((response: TimestampInterval) => {
        this.intervalSelectComponent.instance.animate = false;
        this.intervalSelectComponent.instance.detect();
        if (response) {
          this.router.navigate(this.urlMessageId ? [Routes.NETWORK, Routes.MESSAGES, this.urlMessageId] : [Routes.NETWORK, Routes.MESSAGES], {
            queryParamsHandling: 'merge',
            queryParams: {
              from: response.from / ONE_THOUSAND, to: response.to / ONE_THOUSAND,
            },
          });
          this.dispatch(NetworkMessagesSetTimestampInterval, {
            timestamp: { from: response.from / ONE_THOUSAND, to: response.to / ONE_THOUSAND },
            direction: NetworkMessagesDirection.FORWARD,
          });
        }
        setTimeout(() => this.detachOverlay(), 250);
      });
  }

  detachOverlay(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }

  scrollToTop(): void {
    this.pause();
    this.onScrollTopClick.emit();
  }

  clearTimestampInterval(event: MouseEvent): void {
    event.stopPropagation();
    this.activeInterval = this.currentTimestamp = undefined;
    this.router.navigate(this.urlMessageId ? [Routes.NETWORK, Routes.MESSAGES, this.urlMessageId] : [Routes.NETWORK, Routes.MESSAGES], {
      queryParamsHandling: 'merge',
      queryParams: {
        from: undefined, to: undefined,
      },
    });
    this.dispatch(NetworkMessagesSetTimestampInterval, {
      timestamp: { from: undefined, to: undefined },
      direction: NetworkMessagesDirection.REVERSE,
    });
  }
}
