globalThis.crypto = require("crypto").webcrypto;
import { AptosAccount } from "aptos";
import { bundleAndSignData, createData } from "arbundles";
import { AlgorandSigner, AptosSigner, ArweaveSigner, Signer } from "arbundles/src/signing";
import EthereumSigner from "arbundles/src/signing/chains/ethereumSigner";
import SolanaSigner from "arbundles/src/signing/chains/SolanaSigner";
import keythereum from "keythereum";
import Bundlr from "@bundlr-network/client";
import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";
import fs from "fs";
import crypto from "crypto";
//import algosdk from "algosdk";

const MAX_BUNDLES_AMOUNT = 100;
const MAX_DATA_ITEMS = 100;
const MAX_DATA_BYTES = 1000;
const MAX_APTOS_SIGNERS = 20;

//Arweave
import jwk from "./res/test_wallet.json";

//Algorand
//TODO: fix
//let { sk, addr } = algosdk.generateAccount();

//Ethereum
var params = { keyBytes: 32, ivBytes: 16 };
var { privateKey } = keythereum.create(params);

//Solana
const solKeypair = Keypair.generate();

//Multiaptos
const aptosAccounts = Array.from({ length: Math.ceil(Math.random() * MAX_APTOS_SIGNERS + 1) }, () => new AptosAccount());
const wallet = {
    participants: aptosAccounts.map(a => a.signingKey.publicKey.toString()),
    threshold: 2
}

// create signature collection function
// this function is called whenever the client needs to collect signatures for signing
const collectSignatures = async (message) => {
    //Select random amount of random acccounts within our aptos accounts
    const accountAmount = Math.ceil(Math.random() * aptosAccounts.length);
    const randomAccounts = aptosAccounts
        .map((account, i) => { return { account, i } }) // Store original array position
        .sort(() => Math.random() - Math.random())      // Shuffle array so we get randoms
        .slice(0, accountAmount);                       // Get sample size
    const signatures = randomAccounts.map(el => Buffer.from(el.account.signBuffer(message).toUint8Array()))
    const bitmap = randomAccounts.map(el => el.i);
    return { signatures: signatures, bitmap }
}

const bundlesAmount = MAX_BUNDLES_AMOUNT;

//Create all signers
//TODO: figure out how to instantiate signer directly (see below)
const multiAptosSigner = new Bundlr("https://devnet.bundlr.network", "multiAptos", wallet, { providerUrl: "https://fullnode.devnet.aptoslabs.com", currencyOpts: { collectSignatures } }).getSigner()
const signers: Signer[] = [
    new ArweaveSigner(jwk),
    //new AlgorandSigner(sk, addr),
    new EthereumSigner(privateKey),
    new SolanaSigner(bs58.encode(solKeypair.secretKey)),
    new AptosSigner(aptosAccounts[0].toPrivateKeyObject().privateKeyHex, aptosAccounts[0].toPrivateKeyObject().publicKeyHex),
    //new MultiSignatureAptosSigner(Buffer.from(wallet.participants.join("")), collectSignatures)
    multiAptosSigner
];

for (let i = 1; i <= bundlesAmount; i++) {
    const dataItemsAmount = Math.floor(Math.random() * MAX_DATA_ITEMS + 1);
    const dataItems = [];
    for (let j = 1; j <= dataItemsAmount; j++) {
        const signer = signers[Math.floor(Math.random() * signers.length)];
        const randomData = crypto.randomBytes(MAX_DATA_BYTES).toString('hex');
        try {
            const data = createData(randomData, signer);
            data.sign(signer).then(() => {
                if (data.isSigned()) {
                    dataItems.push(data);
                } else {
                    console.log(`Invalid or unsigned data item: ${data.id}`);
                }
            }).catch(err => {
                console.log(`Error generating data item: ${err}`);
            });
        } catch (err) {
            console.log(err);
        }
    }

    const finalSigner = signers[Math.floor(Math.random() * signers.length)];
    bundleAndSignData(dataItems, finalSigner).then((bundle) => {
        bundle.verify().then(ok => {
            fs.writeFile(`res/gen_bundles/bundle_${i}`, bundle.getRaw(),
                () => console.info(`Generated bundle ${i} in res/gen_bundles/bundle_${i}`));
        }).catch(err => {
            console.log(`Invalid bundle: ${err}`)
        });
    }).catch(err => {
        console.log(`Error generating bundle: ${err}`);
    })
}