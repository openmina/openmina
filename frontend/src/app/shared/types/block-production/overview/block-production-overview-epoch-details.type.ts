export interface BlockProductionOverviewEpochDetails {
  epochNumber: number;
  totalSlots: number;
  wonSlots: number;
  canonical: number;
  orphaned: number;
  missed: number;
  futureRights: number;
  slotStart: number;
  slotEnd: number;
  expectedRewards: string;
  earnedRewards: string;
  balanceDelegated: string;
  balanceProducer: string;
  balanceStaked: string;
}
