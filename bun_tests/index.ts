import { Address, Keypair, hash, nativeToScVal, Networks, Operation, TransactionBuilder, xdr } from "@stellar/stellar-sdk";
import { DEFAULT_TIMEOUT } from "@stellar/stellar-sdk/contract";
import { Api, assembleTransaction, Server } from "@stellar/stellar-sdk/rpc";

const SAC = 'CB23WRDQWGSP6YPMY4UV5C4OW5CBTXKYN3XEATG7KJEZCXMJBYEHOUOV'
const contractID = 'CDL74RF5BLYR2YBLCCI7F5FB6TPSCLKEJUBSD2RSVWZ4YHF3VMFAIGWA'
const networkPassphrase = Networks.PUBLIC;

type TransferMethod = 'transfer' | 'clawback' | 'mint';

const DEFAULT_TO_ADDRESS = 'GD2GA2JF6OJURU36COZQWJLPEJ7XC3GB25TBD7U4ALCGKOG27262RICH';
const rawMethod = process.env.KALE_TRANSFER_METHOD || 'transfer';
if (!['transfer', 'clawback', 'mint'].includes(rawMethod)) {
    throw new Error(`Unsupported KALE_TRANSFER_METHOD: ${rawMethod}`);
}

const method = rawMethod as TransferMethod;
const amount = BigInt(process.env.KALE_TRANSFER_AMOUNT || '0');
const fromAddress = process.env.KALE_TRANSFER_FROM || contractID;
const toAddress = process.env.KALE_TRANSFER_TO || DEFAULT_TO_ADDRESS;

const rpc = new Server("https://mainnet.sorobanrpc.com");

async function loadRequiredSecret(): Promise<string> {
    const secretFile = process.env.KALE_TRANSFER_SECRET_FILE?.trim();
    if (secretFile) {
        const secret = (await Bun.file(secretFile).text()).trim();
        if (!secret) {
            throw new Error(`KALE_TRANSFER_SECRET_FILE is empty: ${secretFile}`);
        }

        return secret;
    }

    const secret = process.env.KALE_TRANSFER_SECRET?.trim();
    if (!secret) {
        throw new Error(
            'Set KALE_TRANSFER_SECRET or KALE_TRANSFER_SECRET_FILE in a gitignored .env file before running this script',
        );
    }

    return secret;
}

const keypair = Keypair.fromSecret(await loadRequiredSecret());
const pubkey = keypair.publicKey(); // GCCX6ZAVF63XCMDFYAT6TPRUWNF3FS43YI6FOJ3JS4MWCYP4QYYJISCV

if (amount <= 0n) {
    throw new Error('KALE_TRANSFER_AMOUNT must be set to a positive integer amount in stroops');
}

async function withRetry<T>(label: string, fn: () => Promise<T>, retries = 8): Promise<T> {
    let lastError: unknown;

    for (let attempt = 1; attempt <= retries; attempt++) {
        try {
            return await fn();
        } catch (error) {
            lastError = error;

            if (attempt === retries) {
                throw error;
            }

            console.error(`retrying ${label} (${attempt}/${retries})`);
            await Bun.sleep(attempt * 500);
        }
    }

    throw lastError;
}

function buildArgs() {
    switch (method) {
        case 'transfer':
            return [
                Address.fromString(fromAddress).toScVal(),
                Address.fromString(toAddress).toScVal(),
                nativeToScVal(amount, { type: 'i128' }),
            ];
        case 'clawback':
            return [
                Address.fromString(fromAddress).toScVal(),
                nativeToScVal(amount, { type: 'i128' }),
            ];
        case 'mint':
            return [
                Address.fromString(toAddress).toScVal(),
                nativeToScVal(amount, { type: 'i128' }),
            ];
    }
}

const acct = await rpc.getAccount(pubkey)
const tx = new TransactionBuilder(acct, {
    fee: (100_000).toString(),
    networkPassphrase
})
.addOperation(Operation.invokeContractFunction({
    contract: SAC,
    function: method,
    args: buildArgs(),
}))
.setTimeout(0)
.build();

const simBefore = await withRetry('simulateTransaction before auth patch', () => rpc.simulateTransaction(tx));

if (
    Api.isSimulationError(simBefore)
    || !simBefore.result
    || !simBefore.result.auth
    || simBefore.result.auth.length === 0
) {
    console.log(simBefore);
} else {
    const entry = xdr.SorobanAuthorizationEntry.fromXDR(simBefore.result.auth[0].toXDR());
    const credentials = entry.credentials().address();
    const lastLedger = await withRetry('getLatestLedger', () => rpc.getLatestLedger()).then(({ sequence }) => sequence);

    credentials.signatureExpirationLedger(lastLedger + DEFAULT_TIMEOUT);
    credentials.signature(xdr.ScVal.scvVoid());

    const signaturePayload = hash(
        xdr.HashIdPreimage.envelopeTypeSorobanAuthorization(
            new xdr.HashIdPreimageSorobanAuthorization({
                networkId: hash(Buffer.from(networkPassphrase)),
                nonce: credentials.nonce(),
                invocation: entry.rootInvocation(),
                signatureExpirationLedger: credentials.signatureExpirationLedger(),
            }),
        ).toXDR(),
    );

    const op = tx.operations[0] as Operation.InvokeHostFunction;

    op.auth?.splice(0, 1, entry);

    const self_invocation = new xdr.InvokeContractArgs({
        contractAddress: Address.fromString(contractID).toScAddress(),
        functionName: "__check_auth",
        args: [xdr.ScVal.scvBytes(signaturePayload)],
    });

    const self_entry = new xdr.SorobanAuthorizationEntry({
        credentials: xdr.SorobanCredentials.sorobanCredentialsSourceAccount(),
        rootInvocation: new xdr.SorobanAuthorizedInvocation({
            function: xdr.SorobanAuthorizedFunction.sorobanAuthorizedFunctionTypeContractFn(self_invocation),
            subInvocations: [],
        }),
    });

    op.auth?.push(self_entry)

    const simAfter = await withRetry('simulateTransaction after auth patch', () => rpc.simulateTransaction(tx));
    if (Api.isSimulationError(simAfter) || !simAfter.result) {
        console.log(simAfter);
        process.exit(1);
    }

    const txAssem = assembleTransaction(tx, simAfter).build();

    txAssem.sign(keypair);

    const sendRes = await withRetry('sendTransaction', () => rpc.sendTransaction(txAssem));
    if (sendRes.status !== 'PENDING') {
        console.log(sendRes);
        process.exit(1);
    }

    const pollRes = await withRetry('pollTransaction', () => rpc.pollTransaction(sendRes.hash));

    if (pollRes.status === 'SUCCESS') {
        console.log(pollRes.status, pollRes.txHash);
    } else if  (pollRes.status === 'NOT_FOUND') {
        console.log(pollRes);
    } else {
        console.log(pollRes.envelopeXdr.toXDR('base64'));
        console.log('\n');
        console.log(pollRes.resultXdr.toXDR('base64'));
        console.log('\n');
        console.log(pollRes.resultMetaXdr.toXDR('base64'));
    }
}
