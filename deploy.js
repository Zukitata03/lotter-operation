import { SigningCosmWasmClient, Secp256k1HdWallet } from 'cosmwasm';
import * as fs from 'fs';
import { Decimal } from '@cosmjs/math';
import * as dotenv from 'dotenv';
import { GasPrice } from '@cosmjs/stargate';

dotenv.config();

// This is your rpc endpoint
const rpcEndpoint = "https://rpc.orai.io";

const mnemonic = process.env.MNEMONIC;
const lotteryContractAddress = process.env.LOTTERY;
const oraiswapRouterAddress = process.env.SWAP;
const contractAddress = process.env.OP;
async function main() {
    try {
        // Load the wallet using mnemonic
        const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: "orai" });
        
        // Connect to the blockchain with gas price settings
        const client = await SigningCosmWasmClient.connectWithSigner(
            rpcEndpoint,
            wallet,
            {
                gasPrice: GasPrice.fromString("0.001orai")
            }
        );

        const account = await wallet.getAccounts();
        const sender = account[0].address;

const balance = await client.getBalance(sender, "orai");
                console.log(`Sender's ORAI balance: ${balance.amount}`);

        // // Path to your contract's compiled .wasm file
        // const path = './artifacts/lottery-operations.wasm';
        // const wasmCode = new Uint8Array(fs.readFileSync(path));

        // // Upload code on chain
        // const upload = await client.upload(sender, wasmCode, "auto");
        // console.log('Upload result:', upload);

        // // Instantiate msg
        // const instantiateMsg = {
        //     owner: sender,
        //     lottery_contract: lotteryContractAddress,
        //     oraiswap_router: oraiswapRouterAddress,
        //     ticket_price: { denom: "USDT", amount: "0" }, // 
        //     round_duration: 600 // 10 min,

        // };

        // // Instantiate the contract
        // const res = await client.instantiate(sender, upload.codeId, instantiateMsg, "Lottery contract", "auto");
        // console.log('Instantiate result:', res);
        // const contractAddress = res.contractAddress;
        // console.log('Contract Address:', contractAddress);



        
        // Query ticket price
        const queryMsg = { get_ticket_price: {} };
        const queryResponse = await client.queryContractSmart(contractAddress, queryMsg);

        // Log ticket price
        console.log('Ticket price:', queryResponse.amount, queryResponse.denom);

        //Test the buy ticket operation
        
        const buyTicketMsg = {
            buy_ticket: {
                amount: "0" 
            }
        };

        const executeRes = await client.execute(sender, contractAddress, buyTicketMsg, "auto");
        console.log('Execute result:', executeRes);



    

        // Disconnect from the client
        await client.disconnect();
    } catch (error) {
        console.error('Error:', error);
    }
}

main();
