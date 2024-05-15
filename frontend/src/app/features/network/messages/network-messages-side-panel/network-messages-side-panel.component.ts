import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, ViewChild } from '@angular/core';
import { NetworkMessagesChangeTab, NetworkMessagesSetActiveRow } from '@network/messages/network-messages.actions';
import { NetworkMessage } from '@shared/types/network/messages/network-message.type';
import { NetworkMessageConnection } from '@shared/types/network/messages/network-messages-connection.type';
import {
  selectNetworkActiveRow,
  selectNetworkConnection,
  selectNetworkFullMessage,
  selectNetworkMessageHex,
} from '@network/messages/network-messages.state';
import { downloadJson, downloadJsonFromURL, ExpandTracking, MinaJsonViewerComponent } from '@openmina/shared';
import { filter } from 'rxjs';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { Clipboard } from '@angular/cdk/clipboard';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppSelectors } from '@app/app.state';

@Component({
  selector: 'mina-network-messages-side-panel',
  templateUrl: './network-messages-side-panel.component.html',
  styleUrls: ['./network-messages-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100 w-100' },
})
export class NetworkMessagesSidePanelComponent extends StoreDispatcher implements AfterViewInit {

  activeRow: NetworkMessage;
  connection: NetworkMessageConnection;
  activeRowFullMessage: any;
  activeRowHexDisplayedValue: string;
  selectedTabIndex: number = 1;
  jsonTooBig: boolean;
  toCopy: string;
  expandingTracking: ExpandTracking = {};

  @ViewChild(MinaJsonViewerComponent) private minaJsonViewer: MinaJsonViewerComponent;
  @ViewChild('saveButton') private saveButton: ElementRef<HTMLButtonElement>;

  private activeRowHex: string;
  private cancelDownload: boolean = false;
  private userDidHitExpandAll: boolean;
  private currentMessageKind: string;
  private debuggerURL: string;

  constructor(private clipboard: Clipboard,
              private router: Router) { super(); }

  ngAfterViewInit(): void {
    this.listenToActiveRowChange();
    this.listenToActiveNodeChange();
  }

  private listenToActiveNodeChange(): void {
    this.select(AppSelectors.activeNode, (node: MinaNode) => {
      this.debuggerURL = node.debugger;
    }, filter(Boolean));
  }

  private listenToActiveRowChange(): void {
    this.select(selectNetworkActiveRow, (activeRow: NetworkMessage) => {
      this.activeRow = activeRow;
      if (activeRow) {
        if (activeRow.messageKind !== this.currentMessageKind) {
          this.expandingTracking = {}; // reset
        }
        this.currentMessageKind = activeRow.messageKind;
      }

      if (!activeRow) {
        this.cancelDownload = true;
        this.saveButton.nativeElement.textContent = 'Save JSON';
        this.activeRowFullMessage = this.activeRowHex = this.activeRowHexDisplayedValue = this.connection = undefined;
      }
      this.detect();
    });

    this.select(selectNetworkFullMessage, (message: any) => {
      this.jsonTooBig = !isNaN(message) ? Number(message) > 10485760 : false;
      this.activeRowFullMessage = message;
      this.setToCopy();
      this.detect();
    }, filter(Boolean));
    this.select(selectNetworkMessageHex, (hex: string) => {
      this.activeRowHex = hex;
      this.activeRowHexDisplayedValue = NetworkMessagesSidePanelComponent.getActiveRowHexDisplayedValue(hex);
      this.setToCopy();
      this.detect();
    }, filter(Boolean));
    this.select(selectNetworkConnection, (connection: NetworkMessageConnection) => {
      this.connection = connection;
      this.setToCopy();
      this.detect();
    }, filter(Boolean));
  }

  private static getActiveRowHexDisplayedValue(hex: string): string {
    return hex.length < 600 ? hex : hex.slice(0, 500) + '...' + hex.slice(hex.length - 9);
  }

  downloadJson(): void {
    const fileName = this.selectedTabIndex === 2 ? 'message_hex.txt' : 'network_data.json';
    if (this.jsonTooBig && this.selectedTabIndex === 1) {
      this.cancelDownload = false;
      const URL = this.debuggerURL + '/message/' + this.activeRow.id;
      downloadJsonFromURL(URL, fileName, () => this.cancelDownload, this.saveButton.nativeElement);
      return;
    }
    const toDownload = this.selectedTabIndex === 1
      ? this.activeRowFullMessage
      : this.selectedTabIndex === 2
        ? this.activeRowHex
        : this.connection;
    downloadJson(toDownload, fileName);
  }

  downloadBinary(): void {
    const fileName = this.activeRow.id + '_binary.bin';
    this.cancelDownload = false;
    const URL = this.debuggerURL + '/message_bin/' + this.activeRow.id;
    downloadJsonFromURL(URL, fileName, () => null);
  }

  private setToCopy(): void {
    this.toCopy = this.selectedTabIndex === 1
      ? JSON.stringify(this.activeRowFullMessage)
      : this.selectedTabIndex === 2
        ? this.activeRowHex
        : JSON.stringify(this.connection);
  }

  closeSidePanel(): void {
    this.router.navigate([Routes.NETWORK, Routes.MESSAGES], { queryParamsHandling: 'merge' });
    this.dispatch(NetworkMessagesSetActiveRow, undefined);
  }

  expandEntireJSON(): void {
    this.userDidHitExpandAll = true;
    this.expandingTracking = this.minaJsonViewer.toggleAll(this.userDidHitExpandAll);
  }

  collapseEntireJSON(): void {
    this.userDidHitExpandAll = false;
    this.expandingTracking = this.minaJsonViewer.toggleAll(this.userDidHitExpandAll);
  }

  selectTab(tabNum: number): void {
    this.cancelDownload = true;
    this.saveButton.nativeElement.textContent = 'Save JSON';
    this.selectedTabIndex = tabNum;
    this.dispatch(NetworkMessagesChangeTab, tabNum);
    this.setToCopy();
  }

  copyToClipboard(): void {
    this.clipboard.copy(window.location.href);
  }
}
