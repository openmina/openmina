import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { TreeNode } from '@network/dht-graph/network-dht-graph-binary-tree/network-dht-graph-binary-tree.component';

export class MinimumDistanceAlgorithm {
  static peers: NetworkNodeDhtPeer[];

  static createBinaryTree(peers: NetworkNodeDhtPeer[]): TreeNode {
    if (peers.length === 0) {
      return null;
    }
    const root: TreeNode = {
      binaryDistance: peers[0].binaryDistance,
      children: [],
    };
    for (let i = 1; i < peers.length; i++) {
      this.insertPeer(root, peers[i], null);
    }
    return this.cleanUpTree(root);
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
        return;
      }
      node.children[direction] = { binaryDistance: peer.binaryDistance, children: [] };
      node.children[1 - direction] = node.children[1 - direction] || {
        binaryDistance: '-',
        children: [],
      };
    } else if (node.children[direction].binaryDistance !== '-') {
      const existingNodePeer = this.peers.find(p => p.binaryDistance === node.children[direction].binaryDistance);
      node.children[direction] = {
        binaryDistance: '-',
        children: [],
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
}
