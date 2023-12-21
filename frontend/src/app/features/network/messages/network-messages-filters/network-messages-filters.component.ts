import { ChangeDetectionStrategy, Component, ElementRef, EventEmitter, OnInit, Output } from '@angular/core';
import {
  selectNetworkActiveFilters,
  selectNetworkTimestampInterval
} from '@network/messages/network-messages.state';
import { NetworkMessagesFilterCategory } from '@shared/types/network/messages/network-messages-filter-group.type';
import { NetworkMessagesToggleFilter } from '@network/messages/network-messages.actions';
import { NetworkMessagesFilter } from '@shared/types/network/messages/network-messages-filter.type';
import { NetworkMessagesFilterTypes } from '@shared/types/network/messages/network-messages-filter-types.enum';
import { ActivatedRoute, Router } from '@angular/router';
import { skip } from 'rxjs';
import { getMergedRoute, MergedRoute, TimestampInterval } from '@openmina/shared';
import { animate, state, style, transition, trigger } from '@angular/animations';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';


export const networkAvailableFilters: NetworkMessagesFilterCategory[][] = [
  [
    {
      name: '/multistream/1.0.0',
      tooltip: '',
      filters: [
        { type: NetworkMessagesFilterTypes.STREAM_KIND, display: 'Multistream', value: '/multistream/1.0.0', tooltip: '' },
      ],
    },
    {
      name: '/coda/mplex/1.0.0',
      tooltip: '',
      filters: [
        { type: NetworkMessagesFilterTypes.STREAM_KIND, display: 'Coda mplex', value: '/coda/mplex/1.0.0', tooltip: '' },
      ],
    },
    {
      name: '/coda/yamux/1.0.0',
      tooltip: '',
      filters: [
        { type: NetworkMessagesFilterTypes.STREAM_KIND, display: 'Coda yamux', value: '/coda/yamux/1.0.0', tooltip: '' },
      ],
    },
  ],
  [
    {
      name: '/noise',
      tooltip: 'The Noise Protocol is a framework for building cryptographic protocols',
      filters: [
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Handshake',
          value: 'handshake_payload',
          tooltip: 'Every Noise connection begins with a handshake between an initiating peer and a responding peer.',
        },
        { type: NetworkMessagesFilterTypes.MESSAGE_KIND, display: 'Failed to decrypt', value: 'failed_to_decrypt', tooltip: '' },
      ],
    },
    {
      name: '/coda/kad/1.0.0',
      tooltip: 'Used for peer discovery. The Kademlia Distributed Hash Table (DHT) is a DHT implementation largely based on the Kademlia whitepaper, augmented with notions from S/Kademlia, Coral and the BitTorrent DHT.',
      filters: [
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Put Value',
          value: 'put_value',
          tooltip: 'Value storage, Putting the value to the closest nodes.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get Value',
          value: 'get_value',
          tooltip: 'Value retrieval, Getting a value by its key from the nodes closest to that key.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Add Provider',
          value: 'add_provider',
          tooltip: 'Adding oneself to the list of providers for a given key at the nodes closest to that key by finding the closest nodes.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get Providers',
          value: 'get_providers',
          tooltip: 'Getting providers for a given key from the nodes closest to that key.',
        },
        { type: NetworkMessagesFilterTypes.MESSAGE_KIND, display: 'Find Node', value: 'find_node', tooltip: 'Finding the closest nodes to a given key.' },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Ping',
          value: 'ping',
          tooltip: 'Deprecated. Message type replaced by the dedicated ping protocol.',
        },
      ],
    },
  ],
  [
    {
      name: '/mina/peer-exchange',
      tooltip: 'Keeps the mesh connected when closing connections.',
      filters: [
        {
          type: NetworkMessagesFilterTypes.STREAM_KIND,
          display: 'Peer Exchange',
          value: '/mina/peer-exchange',
          tooltip: 'Keeps the mesh connected when closing connections.',
        },
      ],
    },
    {
      name: '/mina/bitswap-exchange',
      tooltip: 'Bitswap is a module provided by libp2p that enables distributed data synchronization over a p2p network, somewhat comparable to how BitTorrent works.',
      filters: [
        {
          type: NetworkMessagesFilterTypes.STREAM_KIND,
          display: 'Bitswap Exchange',
          value: '/mina/bitswap-exchange',
          tooltip: 'Using BitSwap, Mina nodes can offset load from GossipSub by gossiping only the header of the block, Splitting the block body into parts and Downloading different parts of the block\'s body.',
        },
      ],
    },
    {
      name: '/mina/node-status',
      tooltip: 'NodeStatus is maintained by libp2p_helper process, which is updated by main Mina process using SetNodeStatus call.',
      filters: [
        {
          type: NetworkMessagesFilterTypes.STREAM_KIND,
          display: 'Node Status',
          value: '/mina/node-status',
          tooltip: 'As soon as you open substream on this protocol, you will receive a json encoded NodeStatus.',
        },
      ],
    },
  ],
  [
    {
      name: '/ipfs/id/1.0.0',
      tooltip: 'The identify protocol has the protocol id /ipfs/id/1.0.0 and it is used to query remote peers for their information.',
      filters: [
        {
          type: NetworkMessagesFilterTypes.STREAM_KIND,
          display: 'Identify',
          value: '/ipfs/id/1.0.0',
          tooltip: 'The protocol works by opening a stream to the remote peer you want to query. The peer being identified responds by returning an Identify message and closing the stream.',
        },
      ],
    },
    {
      name: '/ipfs/id/push/1.0.0',
      tooltip: 'The identify/push protocol has the protocol id /ipfs/id/push/1.0.0 and it is used to inform known peers about changes that occur at runtime.',
      filters: [
        {
          type: NetworkMessagesFilterTypes.STREAM_KIND,
          display: 'IPFS Push',
          value: '/ipfs/id/push/1.0.0',
          tooltip: 'Responds to identify request by pushing the full identity info to the peer.',
        },
      ],
    },
    {
      name: '/p2p/id/delta/1.0.0',
      tooltip: '',
      filters: [
        {
          type: NetworkMessagesFilterTypes.STREAM_KIND,
          display: 'IPFS Delta',
          value: '/p2p/id/delta/1.0.0',
          tooltip: 'Used for providing changes (only diff) in the list of protocols that the peer supports.',
        },
      ],
    },
  ],
  [
    {
      name: '/meshsub/1.1.0',
      tooltip: 'Mina GossipSub is a PubSub (Publish/Subscribe) protocol. Using gossipsub, different messages are published/gossiped through the network. This is used to provide peers with updates regarding the latest state of the blockchain or its network. Each node can, after handshaking and agreeing on this protocol with a peer, publish and subscribe to different topics.',
      filters: [
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Subscribe',
          value: 'subscribe',
          tooltip: 'Peers keep track of which topics their directly connected peers are subscribed to. Using this information each peer is able to build up a picture of the topics around them and which peers are subscribed to each topic.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Unsubscribe',
          value: 'unsubscribe',
          tooltip: 'When a peer unsubscribes from a topic it will notify its full-message peers that their connection has been pruned at the same time as sending their unsubscribe messages.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'iWant',
          value: 'meshsub_iwant',
          tooltip: '',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'iHave',
          value: 'meshsub_ihave',
          tooltip: '',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Prune',
          value: 'meshsub_prune',
          tooltip: '',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Graft',
          value: 'meshsub_graft',
          tooltip: '',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Publish New State',
          value: 'publish_new_state',
          tooltip: 'Broadcasts newly produced blocks throughout the network.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Publish Snark Pool Diff',
          value: 'publish_snark_pool_diff',
          tooltip: 'Broadcasts snark work from mempools throughout the network. Snark coordinator\'s broadcast locally produced snarks on an interval for a period of time after creation, and all nodes rebroadcast externally produced snarks if they were relevant and could be added to the their mempool.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Publish Transaction Pool Diff',
          value: 'publish_transaction_pool_diff',
          tooltip: 'Broadcasts transactions from mempools throughout the network. Nodes broadcast locally submitted transactions on an interval for a period of time after creation, as well as rebroadcast externally submitted transactions if they were relevant and could be added to the their mempool.',
        },
      ],
    },
  ],
  [
    {
      name: 'coda/rpcs/0.0.1',
      tooltip: 'RPC - Remote Procedure Call. In some scenarios, a node needs to obtain information from other nodes it is connected to. It makes a particular request to a peer over the P2P network and then expects a response from it.',
      filters: [
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get some initial peers',
          value: 'get_some_initial_peers',
          tooltip: 'Returns known peers to the requester. Currently used to help nodes quickly connect to the network.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get staged ledger aux and pending coinbases at hash',
          value: 'get_staged_ledger_aux_and_pending_coinbases_at_hash',
          tooltip: 'Returns the staged ledger data associated with a particular block on the blockchain. This is requested by bootstrapping nodes and is used to construct the root staged ledger of the frontier. Subsequent staged ledgers are built from the root using the staged ledger diff information contained in blocks.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Answer sync ledger query',
          value: 'answer_sync_ledger_query',
          tooltip: 'Serves merkle ledger information over a "sync ledger" protocol. The sync ledger protocol works by allowing nodes to request intermediate hashes and leaves of data in a target ledger they want to synchronize with. Using the protocol, requesters are able to determine which parts of their existing ledger need to be updated, and can avoid downloading data in the ledger which is already up to date. Requests are distributed to multiple peers to spread the load of serving sync ledgers throughout the network.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get ancestry',
          value: 'get_ancestry',
          tooltip: 'Returns the target block corresponding to the provided state hash along with the root block of the frontier, and a merkle proof that attests to the connection of blocks linking the target block to the root block.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get best tip',
          value: 'get_best_tip',
          tooltip: 'Returns the best tip block along with the root block of the frontier, and a merkle proof that attests to the connection of blocks linking the target block to the root block. If the root has been finalized, this proof will be of length k-1 to attest to the fact that the best tip block is k blocks on top of the root block.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get node status',
          value: 'get_node_status',
          tooltip: 'This acts as a telemetry RPC which asks a peer to provide information about their node status. Daemons do not have to respond to this request, and node operators may pass a command line flag to opt-out of responding to these node status requests.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get transition chain proof',
          value: 'get_transition_chain_proof',
          tooltip: 'Returns a transition chain proof for a specified block on the blockchain. A transition chain proof proves that, given some block b1, there exists a block b0 which is k blocks preceding b1. To prove this, a node receives H(b1), and returns H(b0) along with a merkle proof of all the intermediate "state body hashes" along the path b0 -> b1. The requester checks this proof by re-computing H(b1) from the provided merkle proof.',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get transition chain',
          value: 'get_transition_chain',
          tooltip: 'Returns a bulk bulk set of blocks associated with a provided set of state hashes. This is used by the catchup routine when it is downloading old blocks to re synchronize with the network over short distances of missing information. At the current moment, the maximum number of blocks that can be requested in a single batch is 20 (requesting more than 20 will result in no response).',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get transition knowledge',
          value: 'get_transition_knowledge',
          tooltip: 'Returns the a list of k state hashes of blocks from the root of the frontier (point of finality) up to the current best tip (most recent block on the canonical chain).',
        },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Get epoch ledger',
          value: 'get_epoch_ledger',
          tooltip: 'Returns sparse ledger, or, in case of a failure, a string description.',
        },
        { type: NetworkMessagesFilterTypes.MESSAGE_KIND, display: 'Versioned rpc menu', value: '__Versioned_rpc.Menu', tooltip: '' },
        {
          type: NetworkMessagesFilterTypes.MESSAGE_KIND,
          display: 'Ban notify',
          value: 'ban_notify',
          tooltip: 'Notifies a peer that they have been banned. The request contains the time at which the requester banned the recipient.',
        },
      ],
    },
  ],
  [
    {
      name: 'unknown',
      tooltip: '',
      filters: [
        { type: NetworkMessagesFilterTypes.STREAM_KIND, display: 'Unknown', value: 'unknown', tooltip: '' },
      ],
    },
  ],
];

@Component({
  selector: 'mina-network-messages-filters',
  templateUrl: './network-messages-filters.component.html',
  styleUrls: ['./network-messages-filters.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  animations: [
    trigger('panelHeight', [
      state('collapsed', style({ height: '36px', overflow: 'hidden' })),
      state('expanded', style({ height: '*' })),
      transition('collapsed <=> expanded', animate('250ms ease-out')),
    ]),
  ],
})
export class NetworkMessagesFiltersComponent extends StoreDispatcher implements OnInit {

  @Output() onSizeChange: EventEmitter<void> = new EventEmitter<void>();

  availableFilters: NetworkMessagesFilterCategory[][] = networkAvailableFilters;
  activeFilters: NetworkMessagesFilter[] = [];
  filtersOpen: boolean;

  private elementHeight: number;
  private timestamp: TimestampInterval;
  private activeNodeName: string;

  constructor(private router: Router,
              private route: ActivatedRoute,
              private elementRef: ElementRef<HTMLElement>) { super(); }

  ngOnInit(): void {
    this.listenToNetworkTimestampFilter();
    this.listenToRouteChange();
    this.listenToNetworkFilters();
  }

  toggleFilerPanel(): void {
    this.filtersOpen = !this.filtersOpen;
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      this.activeNodeName = route.queryParams['node'];
    });
  }

  private listenToNetworkFilters(): void {
    this.select(selectNetworkActiveFilters, (activeFilters: NetworkMessagesFilter[]) => {
      this.activeFilters = activeFilters;
      this.addFiltersToRoute();
      this.detect();
    }, skip(1));
  }

  private addFiltersToRoute(): void {
    const stream_kind = this.activeFilters.filter(f => f.type === NetworkMessagesFilterTypes.STREAM_KIND).map(f => f.value).join(',');
    const message_kind = this.activeFilters.filter(f => f.type === NetworkMessagesFilterTypes.MESSAGE_KIND).map(f => f.value).join(',');
    const addr = this.activeFilters.find(f => f.type === NetworkMessagesFilterTypes.ADDRESS)?.value;
    const queryParams = {
      ...(stream_kind && { stream_kind }),
      ...(message_kind && { message_kind }),
      ...(addr && { addr }),
      ...(this.timestamp?.from && { from: this.timestamp.from, to: this.timestamp.to }),
      ...(this.activeNodeName && { node: this.activeNodeName }),
    };
    this.router.navigate([], {
      relativeTo: this.route,
      queryParams,
    });
  }

  private listenToNetworkTimestampFilter(): void {
    this.select(selectNetworkTimestampInterval, (timestamp: TimestampInterval) => {
      this.timestamp = timestamp;
    });
  }

  toggleFilter(filter: NetworkMessagesFilter): void {
    const type = this.activeFilters.includes(filter) ? 'remove' : 'add';
    const filters = [filter];
    this.dispatch(NetworkMessagesToggleFilter, { filters, type });
    this.onResize();
  }

  onResize(): void {
    if (this.elementHeight !== this.elementRef.nativeElement.offsetHeight) {
      this.elementHeight = this.elementRef.nativeElement.offsetHeight;
      this.onSizeChange.emit();
    }
  }

  filterByCategory(category: NetworkMessagesFilterCategory): void {
    const filters = category.filters;
    const type = filters.every(f => this.activeFilters.includes(f)) ? 'remove' : 'add';
    this.dispatch(NetworkMessagesToggleFilter, { filters, type });
  }
}
