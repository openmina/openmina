export interface BlockProductionOverviewSlot {
  slot: number;
  globalSlot: number;
  height: number;
  time: number;
  status: string;
  finished: boolean;
  canonical: boolean;
  orphaned: boolean;
  missed: boolean;
  futureRights: boolean;
  hash: string;
  active: boolean; // maximum 1 block can be active per epoch
}
