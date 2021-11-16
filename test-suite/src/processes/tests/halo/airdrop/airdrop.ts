import sha256 from 'crypto-js/sha256'
import { MerkleTree } from 'merkletreejs';

class Airdrop {
  private tree: MerkleTree;

  constructor(accounts: Array<{ address: string; amount: string }>) {
    const leaves = accounts.map((a) => sha256(a.address + a.amount));
    this.tree = new MerkleTree(leaves, sha256, { sort: true });
  }

  public getMerkleRoot(): string {
    return this.tree.getHexRoot().replace('0x', '');
  }

  public getMerkleProof(account: {
    address: string;
    amount: string;
  }): string[] {
    return this.tree
      .getHexProof(sha256(account.address + account.amount).toString())
      .map((v) => v.replace('0x', ''));
  }

  public verifyProof(
    proof: string[],
    account: { address: string; amount: string }
  ): boolean {
    const leaf = sha256(account.address + account.amount).toString();
    const root = this.getMerkleRoot();

    const verify = this.tree.verify(proof, leaf, root);
    return verify;
  }
}

export {Airdrop}
