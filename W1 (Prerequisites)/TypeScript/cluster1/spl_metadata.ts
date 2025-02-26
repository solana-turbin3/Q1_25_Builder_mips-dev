import wallet from "./wba-wallet.json"
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import { 
    createMetadataAccountV3, 
    CreateMetadataAccountV3InstructionAccounts, 
    CreateMetadataAccountV3InstructionArgs,
    DataV2Args,
    MPL_TOKEN_METADATA_PROGRAM_ID
} from "@metaplex-foundation/mpl-token-metadata";
import { createSignerFromKeypair, signerIdentity, publicKey } from "@metaplex-foundation/umi";
import { PublicKey } from "@solana/web3.js";

// Define our Mint address
const mint = publicKey("CG91n7mgzs8waJDLo7SR4X89DpmGxxoqbKEkd12msNEs")

// Create a UMI connection
const umi = createUmi('https://api.devnet.solana.com');
const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(createSignerFromKeypair(umi, keypair)));

(async () => {
    try {
        // Start here
         let accounts: CreateMetadataAccountV3InstructionAccounts = {
            mint, mintAuthority:signer
        }

        let data: DataV2Args = {
            name:"w",
            symbol:"$w",
            uri:"",
            sellerFeeBasisPoints: 0,
            creators:null,
            collection:null,
            uses:null
        }

        let args: CreateMetadataAccountV3InstructionArgs = {
            data,
            isMutable: true,
            collectionDetails:null
        }

        let tx = createMetadataAccountV3(
             umi,
             {
                 ...accounts,
                 ...args
             }
        )

        let result = await tx.sendAndConfirm(umi);
        console.log(umi.transactions.deserialize(result.signature));
    } catch(e) {
        console.error(`Oops, something went wrong: ${e}`)
    }
})();
