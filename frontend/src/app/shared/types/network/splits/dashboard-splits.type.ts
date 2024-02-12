import { DashboardSplitsPeer } from '@shared/types/network/splits/dashboard-splits-peer.type';
import { DashboardSplitsLink } from '@shared/types/network/splits/dashboard-splits-link.type';

export interface DashboardSplits {
  peers: DashboardSplitsPeer[];
  links: DashboardSplitsLink[];
}
