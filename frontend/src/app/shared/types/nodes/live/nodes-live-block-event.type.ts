export interface NodesLiveBlockEvent {
  datetime: string;
  timestamp: number;
  height: number;
  message: string;
  status: string;
  elapsed: number;
  isBestTip: boolean;
}
