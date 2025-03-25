import { Injectable } from '@angular/core';
import { concatAll, delay, forkJoin, from, map, Observable, tap, toArray } from 'rxjs';
import { HttpClient } from '@angular/common/http';
import { DashboardSplits } from '@shared/types/network/splits/dashboard-splits.type';
import { CONFIG } from '@shared/constants/config';
import { DashboardSplitsPeer } from '@shared/types/network/splits/dashboard-splits-peer.type';
import { DashboardSplitsLink } from '@shared/types/network/splits/dashboard-splits-link.type';

@Injectable({ providedIn: 'root' })
export class DashboardSplitsService {

  private readonly options = { headers: { 'Content-Type': 'application/json' } };

  private currentData: DashboardSplits;

  constructor(private http: HttpClient) { }

  getPeers(): Observable<DashboardSplits> {
    return from(
      [].map(node =>
        this.http
          .post<{ data: GetPeersResponse }>(`${node.graphql}/graphql`, { query: peersQuery }, this.options)
          .pipe(
            map(response => ({ data: response.data, node: node.name })),
          ),
      ),
    ).pipe(
      delay(100),
      concatAll(),
      toArray(),
      /* This is how the data was coming from the backend */
      // map(() => peersMock),
      // map((array/*: { data: GetPeersResponse, node: string }[]*/) => {
      //   const map = new Map<string, string>();
      //   array.forEach(({ data, node }: { data: GetPeersResponse, node: string }) => {
      //     node = node.charAt(0).toUpperCase() + node.slice(1);
      //     node = node.replace(/(\d+)/g, ' $1');
      //     map.set(data.daemonStatus.addrsAndPorts.externalIp, node);
      //   });
      //   return array.reduce((acc: DashboardSplits, { data }: { data: GetPeersResponse }) => {
      //     (acc as any).peers.push(
      //       ...[
      //         ...data.getPeers
      //           .slice(0, 10)
      //           .map((p: GetPeers) => ({ address: p.host, /*peerId: p.peerId,*/ node: map.get(p.host) })),
      //         {
      //           address: data.daemonStatus.addrsAndPorts.externalIp,
      //           // peerId: data.daemonStatus.addrsAndPorts.peer.peerId,
      //           node: map.get(data.daemonStatus.addrsAndPorts.externalIp),
      //         },
      //         {
      //           address: 'rand',
      //           node: 'seed16',
      //         },
      //         {
      //           address: 'rand2',
      //           node: 'seed15',
      //         },
      //       ],
      //     );
      //     acc.links.push(
      //       ...data.getPeers
      //         .slice(0, 10)
      //         .map((p: GetPeers) => ({
      //           source: data.daemonStatus.addrsAndPorts.externalIp,
      //           target: p.host,
      //         })),
      //     );
      //     acc.links.push(
      //       {
      //         source: 'rand2',
      //         target: 'rand',
      //       }
      //     )
      //     return acc;
      //   }, { peers: new Array<DashboardSplitsPeer>(), links: new Array<DashboardSplitsLink>() });
      // }),
      // map((response: DashboardSplits) => this.removeDuplicatedPeers(response)),
      map(() => {
        // Helper function to generate random IP addresses
        function generateRandomIpAddress(): string {
          const octet1 = Math.floor(Math.random() * 256);
          const octet2 = Math.floor(Math.random() * 256);
          const octet3 = Math.floor(Math.random() * 256);
          const octet4 = Math.floor(Math.random() * 256);

          return `${octet1}.${octet2}.${octet3}.${octet4}`;
        }

        /**
         * Generates a network of peers with random connections
         * @param peerCount The number of peers to create (default: 10)
         * @param connectionsPerPeer The number of random connections per peer (default: 5)
         * @returns A DashboardSplits object containing peers and their connections
         */
        function generatePeerNetwork(peerCount: number = 100, connectionsPerPeer: number = 50): DashboardSplits {
          // Create array to hold our peers
          const peers: DashboardSplitsPeer[] = [];

          // Generate peers with random addresses and names
          for (let i = 0; i < peerCount; i++) {
            const address = generateRandomIpAddress();
            const nodeName = i === 0 ? 'Webnode' : `Node-${i + 1}`;
            peers.push({ address, node: nodeName });
          }

          // Create array to hold our links
          const links: DashboardSplitsLink[] = [];

          // Create a set to track unique connections and avoid duplicates
          const connectionSet = new Set<string>();

          // For each peer, create random connections
          peers.forEach(peer => {
            // We want to create exactly connectionsPerPeer links if possible
            let connectionsMade = 0;
            let attempts = 0;
            const maxAttempts = peerCount * 2; // Prevent infinite loops

            while (connectionsMade < connectionsPerPeer && attempts < maxAttempts) {
              attempts++;

              // Choose a random target peer that is not the current peer
              const randomIndex = Math.floor(Math.random() * peerCount);
              const targetPeer = peers[randomIndex];

              if (targetPeer.address === peer.address) {
                continue; // Skip self-connections
              }

              // Create a unique identifier for this connection (ordered by address to avoid duplicates)
              const sourceAddr = peer.address;
              const targetAddr = targetPeer.address;

              // Sort addresses to make sure we don't add the same connection twice in different directions
              const [addr1, addr2] = [sourceAddr, targetAddr].sort();
              const connectionKey = `${addr1}-${addr2}`;

              // Check if this connection already exists
              if (!connectionSet.has(connectionKey)) {
                // Add the connection
                links.push({
                  source: sourceAddr,
                  target: targetAddr
                });

                // Mark this connection as used
                connectionSet.add(connectionKey);
                connectionsMade++;
              }
            }
          });

          return {
            peers,
            links
          };
        }

        if (this.currentData) {
          return { ...this.currentData };
        }

        this.currentData = generatePeerNetwork();
        return this.currentData;
      }),
      tap((d) => console.log(d))
    );
  }

  splitNodes(_peers?: DashboardSplitsPeer[]): void {
    // First, identify the existing disconnected components in the network
    const components = this.identifyDisconnectedComponents();

    // Sort components by size (largest first) to make sure we split the largest groups
    components.sort((a, b) => b.length - a.length);

    // Find a group large enough to split (minimum 4 peers to create two groups of at least 2)
    const groupToSplitIndex = components.findIndex(group => group.length >= 4);

    // If no group is large enough to split, return the current data unchanged
    if (groupToSplitIndex === -1) {
      console.log('No groups large enough to split further.');
      this.currentData = {
        peers: [...this.currentData.peers],
        links: [...this.currentData.links]
      };
    }

    // Get the group we'll split
    const groupToSplit = components[groupToSplitIndex];

    if (!groupToSplit) {
      return;
    }

    // Determine split point - approximately half
    const splitIndex = Math.floor(groupToSplit.length / 2);

    // Separate the group into two subgroups
    const subgroup1 = groupToSplit.slice(0, splitIndex);
    const subgroup2 = groupToSplit.slice(splitIndex);

    // Create sets of addresses for faster lookups
    const subgroup1Addresses = new Set<string>(subgroup1);
    const subgroup2Addresses = new Set<string>(subgroup2);

    // Filter out links that connect between the two subgroups
    const newLinks = this.currentData.links.filter(link => {
      // If both endpoints are in the group we're splitting...
      if (groupToSplit.includes(link.source) && groupToSplit.includes(link.target)) {
        // Keep only if both are in the same subgroup
        return (
          (subgroup1Addresses.has(link.source) && subgroup1Addresses.has(link.target)) ||
          (subgroup2Addresses.has(link.source) && subgroup2Addresses.has(link.target))
        );
      }
      // Keep all links not related to the group we're splitting
      return true;
    });

    // Ensure each subgroup is connected (has sufficient internal links)
    const additionalLinks = [
      ...this.ensureGroupIsConnected(subgroup1, this.currentData.peers),
      ...this.ensureGroupIsConnected(subgroup2, this.currentData.peers)
    ];

    // Create a new object with the updated links
    this.currentData = {
      peers: [...this.currentData.peers],
      links: [...newLinks, ...additionalLinks]
    };
  }

  /**
   * Identifies disconnected components (clusters) in the network
   * @returns Array of arrays, where each inner array contains the addresses of peers in one connected component
   */
  private identifyDisconnectedComponents(): string[][] {
    // Create an adjacency map
    const adjacencyMap = new Map<string, Set<string>>();

    // Initialize the map with all peers
    this.currentData.peers.forEach(peer => {
      adjacencyMap.set(peer.address, new Set<string>());
    });

    // Populate the adjacency map
    this.currentData.links.forEach(link => {
      adjacencyMap.get(link.source)?.add(link.target);
      adjacencyMap.get(link.target)?.add(link.source);
    });

    // Set to track visited nodes
    const visited = new Set<string>();

    // Array to hold the components
    const components: string[][] = [];

    // Function to perform depth-first search
    const dfs = (node: string, component: string[]) => {
      visited.add(node);
      component.push(node);

      const neighbors = adjacencyMap.get(node) || new Set<string>();
      for (const neighbor of neighbors) {
        if (!visited.has(neighbor)) {
          dfs(neighbor, component);
        }
      }
    };

    // Find all components
    for (const peer of this.currentData.peers) {
      if (!visited.has(peer.address)) {
        const component: string[] = [];
        dfs(peer.address, component);
        components.push(component);
      }
    }

    return components;
  }

  /**
   * Ensures a group of peers is connected by adding links if necessary
   * @param group Array of peer addresses in the group
   * @param allPeers All peers in the network
   * @returns Array of new links needed to ensure connectivity
   */
  private ensureGroupIsConnected(group: string[], allPeers: DashboardSplitsPeer[]): DashboardSplitsLink[] {
    // If the group has fewer than 2 peers, it can't be connected
    if (group.length < 2) {
      return [];
    }

    // Create a set of addresses for faster lookups
    const groupAddresses = new Set<string>(group);

    // Find existing links within this group
    const groupLinks = this.currentData.links.filter(link =>
      groupAddresses.has(link.source) && groupAddresses.has(link.target)
    );

    // If there are already links, we need to check if the group is connected
    if (groupLinks.length > 0) {
      // Use a simplified connectivity check - make sure there's a path to every node
      const connectivity = this.checkConnectivity(group, groupLinks);

      if (connectivity.fullyConnected) {
        return []; // Already connected, no new links needed
      }

      // If not connected, we need to add links between disconnected subgroups
      const newLinks: DashboardSplitsLink[] = [];

      // We'll connect each isolated component to the largest component
      for (let i = 1; i < connectivity.components.length; i++) {
        const sourceNode = connectivity.components[0][0]; // Node from largest component
        const targetNode = connectivity.components[i][0]; // Node from isolated component

        newLinks.push({
          source: sourceNode,
          target: targetNode
        });
      }

      return newLinks;
    } else {
      // No existing links in this group - create a minimal spanning tree
      const newLinks: DashboardSplitsLink[] = [];

      // Simple approach: connect each node to the next one in sequence
      for (let i = 0; i < group.length - 1; i++) {
        newLinks.push({
          source: group[i],
          target: group[i + 1]
        });
      }

      return newLinks;
    }
  }

  /**
   * Checks the connectivity of a group of peers
   * @param group Array of peer addresses in the group
   * @param groupLinks Existing links within the group
   * @returns Object containing connectivity information
   */
  private checkConnectivity(group: string[], groupLinks: DashboardSplitsLink[]): {
    fullyConnected: boolean;
    components: string[][]
  } {
    // Create an adjacency map
    const adjacencyMap = new Map<string, Set<string>>();

    // Initialize the map with all peers in the group
    group.forEach(address => {
      adjacencyMap.set(address, new Set<string>());
    });

    // Populate the adjacency map
    groupLinks.forEach(link => {
      adjacencyMap.get(link.source)?.add(link.target);
      adjacencyMap.get(link.target)?.add(link.source);
    });

    // Set to track visited nodes
    const visited = new Set<string>();

    // Array to hold the components
    const components: string[][] = [];

    // Function to perform depth-first search
    const dfs = (node: string, component: string[]) => {
      visited.add(node);
      component.push(node);

      const neighbors = adjacencyMap.get(node) || new Set<string>();
      for (const neighbor of neighbors) {
        if (!visited.has(neighbor)) {
          dfs(neighbor, component);
        }
      }
    };

    // Find all components
    for (const address of group) {
      if (!visited.has(address)) {
        const component: string[] = [];
        dfs(address, component);
        components.push(component);
      }
    }

    // The group is fully connected if there's only one component
    const fullyConnected = components.length === 1;

    return { fullyConnected, components };
  }

  splitNodesOld(peers: DashboardSplitsPeer[]): Observable<void> {
    const nodeName = (peer: DashboardSplitsPeer) => peer.node.toLowerCase().replace(' ', '');
    const lastChar = (str: string) => str[str.length - 1];
    const leftList = peers.filter(p => p.node).filter(p => Number(lastChar(p.node)) % 2 === 0);
    const rightList = peers.filter(p => p.node).filter(p => Number(lastChar(p.node)) % 2 !== 0);

    const leftObs = from(
      leftList.map(node =>
        this.http
          .post<any>(CONFIG.configs.find(c => c.name === nodeName(node)).debugger + '/firewall/whitelist/enable', {
            'ips': leftList.map(n => n.address.split(':')[0]),
            'ports': [10909, 10001],
          }, this.options),
      ),
    ).pipe(
      concatAll(),
      toArray(),
    );
    const rightObs = from(
      rightList.map(node =>
        this.http
          .post<any>(CONFIG.configs.find(c => c.name === nodeName(node)).debugger + '/firewall/whitelist/enable', {
            'ips': rightList.map(n => n.address.split(':')[0]),
            'ports': [10909, 10001],
          }, this.options),
      ),
    ).pipe(
      concatAll(),
      toArray(),
    );

    return forkJoin([leftObs, rightObs]).pipe(map(() => void 0));
  }

  mergeNodes(_: any): void {
    // Identify the existing disconnected components in the network
    const components = this.identifyDisconnectedComponents();

    // If there's only one component, the network is already fully connected
    if (components.length <= 1) {
      console.log('Network is already fully connected.');
      this.currentData = {
        peers: [...this.currentData.peers],
        links: [...this.currentData.links]
      };
    }

    // Start with the existing links
    const newLinks = [...this.currentData.links];

    // Connect each component to the next one to form a "chain" of components
    for (let i = 0; i < components.length - 1; i++) {
      // Get a random node from the current component
      const sourceNode = components[i][Math.floor(Math.random() * components[i].length)];

      // Get a random node from the next component
      const targetNode = components[i + 1][Math.floor(Math.random() * components[i + 1].length)];

      // Add a link between them
      newLinks.push({
        source: sourceNode,
        target: targetNode
      });
    }

    // Create a new object with the updated links
    this.currentData = {
      peers: [...this.currentData.peers],
      links: newLinks
    };
  }

  mergeNodesOld(peers: DashboardSplitsPeer[]): Observable<void> {
    const nodeName = (peer: DashboardSplitsPeer) => peer.node.toLowerCase().replace(' ', '');

    return from(
      peers.filter(p => p.node).map(node =>
        this.http.post<void>(
          CONFIG.configs.find(c => c.name === nodeName(node)).debugger + '/firewall/whitelist/disable', null, this.options,
        ),
      ),
    ).pipe(
      concatAll(),
      toArray(),
      map(() => void 0),
    );
  }

  private removeDuplicatedPeers(response: DashboardSplits): DashboardSplits {
    response.peers = response.peers.filter((peer: DashboardSplitsPeer, index: number, peers: DashboardSplitsPeer[]) =>
      index === peers.findIndex(p => p.address === peer.address),
    );
    return response;
  }
}

interface GetPeersResponse {
  getPeers: GetPeers[];
  daemonStatus: DaemonStatus;
}

interface GetPeers {
  host: string;
  libp2pPort: number;
  peerId: string;
}

interface DaemonStatus {
  addrsAndPorts: AddrsAndPorts;
}

interface AddrsAndPorts {
  externalIp: string;
  libp2pPort: number;
  peer: { peerId: string; };
}

const peersQuery = `
  query Peers {
    daemonStatus {
      addrsAndPorts {
        peer {
          peerId
        }
        externalIp
        libp2pPort
      }
    }
    getPeers {
      host
      libp2pPort
      peerId
    }
  }
`;
