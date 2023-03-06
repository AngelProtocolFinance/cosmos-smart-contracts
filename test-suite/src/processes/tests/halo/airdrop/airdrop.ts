import sha256 from "crypto-js/sha256";
import { MerkleTree } from "merkletreejs";

class Airdrop {
    private tree: MerkleTree;

    constructor(accounts: Array<{ address: string; amount: string }>) {
        const leaves = accounts.map((a) => sha256(a.address + a.amount));
        this.tree = new MerkleTree(leaves, sha256, { sort: true });
    }

    public getMerkleRoot(): string {
        return this.tree.getHexRoot().replace("0x", "");
    }

    public getMerkleProof(account: { address: string; amount: string }): string[] {
        return this.tree
            .getHexProof(sha256(account.address + account.amount).toString())
            .map((v) => v.replace("0x", ""));
    }

    public verify(proof: string[], account: { address: string; amount: string }): boolean {
        console.log(this.tree.toString());
        let hashBuf = Buffer.from(sha256(account.address + account.amount).toString(), "hex");
        console.log(hashBuf.toString("hex"));

        proof.forEach((proofElem) => {
            const proofBuf = Buffer.from(proofElem, "hex");
            console.log("proofBuf", proofBuf.toString("hex"));
            if (Buffer.compare(hashBuf, proofBuf) < 0) {
                hashBuf = Buffer.from(sha256(Buffer.concat([hashBuf, proofBuf]).toString()).toString(), "hex");
            } else {
                hashBuf = Buffer.from(sha256(Buffer.concat([proofBuf, hashBuf]).toString()).toString(), "hex");
            }
            console.log("hashBuf", hashBuf.toString("hex"));
        });
        return this.getMerkleRoot() === hashBuf.toString("hex");
    }
}

export { Airdrop };
