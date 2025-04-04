export interface HeartbeatSummary {
  publicKey: string;
  isWhale: boolean;
  uptimePercentage: number;
  blocksProduced: number;
  uptimePrize: boolean;
  blocksPrize: boolean;
  score: number;
  maxScore: number;
}
