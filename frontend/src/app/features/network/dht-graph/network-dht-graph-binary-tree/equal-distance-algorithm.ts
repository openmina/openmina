import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { TreeNode } from '@network/dht-graph/network-dht-graph-binary-tree/network-dht-graph-binary-tree.component';

export class EqualDistanceAlgorithm {
  static peers: NetworkNodeDhtPeer[];
  private static maxDistance: number = 0;
  static id = 0;

  static createBinaryTree(peers: NetworkNodeDhtPeer[]): TreeNode {
    if (peers.length === 0) {
      return null;
    }
    let root: TreeNode = {
      binaryDistance: peers[0].binaryDistance,
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
    console.log(root);
    return root;
  }

  private static insertPeer(node: TreeNode, peer: NetworkNodeDhtPeer, parentOfNode: TreeNode, level: number = 0): void {
    if (level > peer.binaryDistance.length) {
      return;
    }
    const direction = peer.binaryDistance[level] === '0' ? 0 : 1;

    if (!node.children[direction]) {
      if (
        peer.binaryDistance[level - 1] === '1'
        && parentOfNode.children[0]
        && parentOfNode.children[0].children.length === 0
        && parentOfNode.children[0].binaryDistance !== '-'
        && node.children.length === 0
        && parentOfNode.children[0].binaryDistance.slice(0, level - 1) === peer.binaryDistance.slice(0, level - 1)
        && this.peers.slice(this.peers.indexOf(peer) + 1).every(p => p.binaryDistance.slice(0, level - 1) !== peer.binaryDistance.slice(0, level - 1))
      ) {
        node.binaryDistance = peer.binaryDistance;
        node.children = [];
        node.id = this.id++;
        node.prevBranch = parentOfNode.children.indexOf(node) as 0 | 1;
        return;
      }
      node.children[direction] = {
        binaryDistance: peer.binaryDistance,
        children: [],
        id: this.id++,
        prevBranch: direction,
      };
      node.children[1 - direction] = node.children[1 - direction] || {
        binaryDistance: '-',
        children: [],
        id: this.id++,
        prevBranch: 1 - direction as 0 | 1,
      };
    } else if (node.children[direction].binaryDistance !== '-') {
      const existingNodePeer = this.peers.find(p => p.binaryDistance === node.children[direction].binaryDistance);
      node.children[direction] = {
        binaryDistance: '-',
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
  private static cleanUpTree(node: TreeNode): TreeNode {
    if (node.children.length === 0) {
      return node;
    }
    if (node.children[0].binaryDistance === '-' && node.children[1].binaryDistance !== '-') {
      node.binaryDistance = node.children[1].binaryDistance;
      node.children = [];
      return node;
    }
    node.children[0] = this.cleanUpTree(node.children[0]);
    node.children[1] = this.cleanUpTree(node.children[1]);
    return node;
  }

  private static findMaxDistance(node: TreeNode, distance: number): void {
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
  private static makeEqualDistancesForAllNodes(node: TreeNode, parentNode: TreeNode, distance: number): void {
    if (distance === this.maxDistance) {
      return;
    }
    if (distance < this.maxDistance) {
      if (node.children.length === 0) {
        // check the next digit in the binary distance based on current distance
        if (node.binaryDistance[distance] === '0') {
          node.children[0] = { binaryDistance: node.binaryDistance, children: [], prevBranch: 0 };
          // node.children[1] = { binaryDistance: '-', children: [] };
          node.binaryDistance = '-';
        } else if (node.binaryDistance[distance] === '1') {
          // node.children[0] = { binaryDistance: '-', children: [] };
          node.children[0] = { binaryDistance: node.binaryDistance, children: [], prevBranch: 1 };
          node.binaryDistance = '-';
        } else if (node.binaryDistance === '-') {
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

  private static removeEmptyNodes(node: TreeNode, parentNode: TreeNode): void {
    if (!node) {
      return;
    }
    if (node.children.length === 0 && node.binaryDistance === '-') {
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
