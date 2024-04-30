export interface BlockProductionOverviewAllStats {
  wonSlots: number;
  totalSlots: number;
  canonical: number;
  orphaned: number;
  missed: number;
  futureRights: number;
  expectedRewards: string;
  earnedRewards: string;
}
