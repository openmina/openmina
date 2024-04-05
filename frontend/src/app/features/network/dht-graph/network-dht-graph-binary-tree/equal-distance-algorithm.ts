import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { DhtGraphNode } from '@network/dht-graph/network-dht-graph-binary-tree/network-dht-graph-binary-tree.component';

export class EqualDistanceAlgorithm {
  static peers: NetworkNodeDhtPeer[];
  private static maxDistance: number = 0;
  static id = 0;

  static createBinaryTree(peers: NetworkNodeDhtPeer[]): DhtGraphNode {
    if (peers.length === 0) {
      return null;
    }
    let root: DhtGraphNode = {
      peer: peers[0],
      children: [],
      prevBranch: null,
      root: true,
    } as any;
    for (let i = 1; i < peers.length; i++) {
      this.insertPeer(root, peers[i], null);
    }
    root = this.cleanUpTree(root);
    this.findMaxDistance(root, 0);
    this.makeEqualDistancesForAllNodes(root, null, 0);
    this.removeEmptyNodes(root, null);
    return root;
  }

  private static insertPeer(node: DhtGraphNode, peer: NetworkNodeDhtPeer, parentOfNode: DhtGraphNode, level: number = 0): void {
    if (level > peer.binaryDistance.length) {
      return;
    }
    const direction = peer.binaryDistance[level] === '0' ? 0 : 1;

    if (!node.children[direction]) {
      if (
        peer.binaryDistance[level - 1] === '1'
        && parentOfNode.children[0]
        && parentOfNode.children[0].children.length === 0
        && parentOfNode.children[0].peer.binaryDistance !== '-'
        && node.children.length === 0
        && parentOfNode.children[0].peer.binaryDistance.slice(0, level - 1) === peer.binaryDistance.slice(0, level - 1)
        && this.peers.slice(this.peers.indexOf(peer) + 1).every(p => p.binaryDistance.slice(0, level - 1) !== peer.binaryDistance.slice(0, level - 1))
      ) {
        node.peer = peer;
        node.children = [];
        node.id = this.id++;
        node.prevBranch = parentOfNode.children.indexOf(node) as 0 | 1;
        return;
      }
      node.children[direction] = {
        peer,
        children: [],
        id: this.id++,
        prevBranch: direction,
      };
      node.children[1 - direction] = node.children[1 - direction] || {
        peer: { binaryDistance: '-' } as NetworkNodeDhtPeer,
        children: [],
        id: this.id++,
        prevBranch: 1 - direction as 0 | 1,
      };
    } else if (node.children[direction].peer.binaryDistance !== '-') {
      const existingNodePeer = this.peers.find(p => p.binaryDistance === node.children[direction].peer.binaryDistance);
      node.children[direction] = {
        peer: { binaryDistance: '-' } as NetworkNodeDhtPeer,
        children: [],
        id: this.id++,
        prevBranch: direction as 0 | 1,
      };
      this.insertPeer(node.children[direction], existingNodePeer, node, level + 1);
      this.insertPeer(node.children[direction], peer, node, level + 1);
    } else {
      this.insertPeer(node.children[direction], peer, node, level + 1);
    }
  }

  /**
   * traverse the tree and every time we find a node where
   * it's sibling is binaryDistance: '-' and its parent's binaryDistance is '-' then
   * we set the parent's binaryDistance to the node's binaryDistance and remove all children underneath it
   */
  private static cleanUpTree(node: DhtGraphNode): DhtGraphNode {
    if (node.children.length === 0) {
      return node;
    }
    if (node.children[0].peer.binaryDistance === '-' && node.children[1].peer.binaryDistance !== '-') {
      node.peer = node.children[1].peer;
      node.children = [];
      return node;
    }
    node.children[0] = this.cleanUpTree(node.children[0]);
    node.children[1] = this.cleanUpTree(node.children[1]);
    return node;
  }

  private static findMaxDistance(node: DhtGraphNode, distance: number): void {
    if (node.children.length === 0) {
      this.maxDistance = Math.max(this.maxDistance, distance);
      return;
    }
    this.findMaxDistance(node.children[0], distance + 1);
    this.findMaxDistance(node.children[1], distance + 1);
  }

  /**
   * This function will make sure that all nodes have this.maxDistance distance from the root
   * so that the binary tree is balanced.
   *
   * @private
   */
  private static makeEqualDistancesForAllNodes(node: DhtGraphNode, parentNode: DhtGraphNode, distance: number): void {
    if (distance === this.maxDistance) {
      return;
    }
    if (distance < this.maxDistance) {
      if (node.children.length === 0) {
        // check the next digit in the binary distance based on current distance
        if (node.peer.binaryDistance[distance] === '0') {
          node.children[0] = { peer: node.peer, children: [], prevBranch: 0 };
          // node.children[1] = { binaryDistance: '-', children: [] };
          node.peer = { binaryDistance: '-' } as NetworkNodeDhtPeer;
        } else if (node.peer.binaryDistance[distance] === '1') {
          // node.children[0] = { binaryDistance: '-', children: [] };
          node.children[0] = { peer: node.peer, children: [], prevBranch: 1 };
          node.peer = { binaryDistance: '-' } as NetworkNodeDhtPeer;
        } else if (node.peer.binaryDistance === '-') {
          // parentNode.children = parentNode.children.filter(n => n !== node);
          // node.binaryDistance = null;
          return;
          // node.children[0] = { binaryDistance: '-', children: [] };
          // node.children[1] = { binaryDistance: '-', children: [] };
        }
      } else {
        node.prevBranch = parentNode?.children.indexOf(node) as 0 | 1;
      }
    }
    if (node.children[0]) {
      this.makeEqualDistancesForAllNodes(node.children[0], node, distance + 1);
    }
    if (node.children[1]) {
      this.makeEqualDistancesForAllNodes(node.children[1], node, distance + 1);
    }
  }

  private static removeEmptyNodes(node: DhtGraphNode, parentNode: DhtGraphNode): void {
    if (!node) {
      return;
    }
    if (node.children.length === 0 && node.peer.binaryDistance === '-') {
      const index = parentNode.children.indexOf(node);
      if (index !== -1) {
        parentNode.children.splice(index, 1);
      }
    }
    const hadChildIndex1 = !!node.children[1];
    this.removeEmptyNodes(node.children[0], node);
    if (node.children[1]) {
      this.removeEmptyNodes(node.children[1], node);
    } else if (hadChildIndex1 && !node.children[1]) {
      // child 0 was removed, so el at index 1 is now at index 0
      this.removeEmptyNodes(node.children[0], node);
    }
  }
}
