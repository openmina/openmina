export interface DashboardPeer {
  peerId: string;
  status: DashboardPeerStatus;
  datetime: string;
  timestamp: number;
  address: string | null;
  bestTip: string | null;
  globalSlot: number;
  bestTipTimestamp: number;
  bestTipDatetime: string;
  height: number;
}

export enum DashboardPeerStatus {
  CONNECTED = 'Connected',
  CONNECTING = 'Connecting',
  DISCONNECTED = 'Disconnected',
}
