import * as admin from 'firebase-admin';
import * as functions from 'firebase-functions';
import * as blake2 from 'blake2';
import bs58check from 'bs58check';
import Client from 'mina-signer';
import { submitterAllowed } from './submitterValidator';
import { CallableRequest, onCall } from 'firebase-functions/v2/https';
import { getFirestore, FieldValue } from 'firebase-admin/firestore';

interface SignatureJson {
    field: string;
    scalar: string;
}

interface HeartbeatData {
    version: number;
    payload: string;
    submitter: string;
    signature: SignatureJson;
}

const minaClient = new Client({ network: 'testnet' });

admin.initializeApp();

// Rate limit configuration: sliding window
const WINDOW_SIZE_MS = 60000; // 1 minute window
const MAX_REQUESTS_PER_WINDOW = 6;

function validateSignature(
    data: string,
    signature: SignatureJson,
    publicKeyBase58: string,
): boolean {
    try {
        const h = blake2.createHash('blake2b', { digestLength: 32 });
        h.update(Buffer.from(data));
        const digest: string = h.digest().toString('hex');

        try {
            // TODO: remove this validation later, since the list is
            // hardcoded and we check that the key is there,
            // we know it is valid.
            let publicKeyBytes: Uint8Array;
            try {
                publicKeyBytes = bs58check.decode(publicKeyBase58);
            } catch (e) {
                console.error('Failed to decode public key:', e);
                return false;
            }

            if (publicKeyBytes[0] !== 0xcb) {
                console.error('Invalid public key prefix');
                return false;
            }

            return minaClient.verifyMessage({
                data: digest,
                signature,
                publicKey: publicKeyBase58,
            });
        } catch (e) {
            console.error('Error parsing signature or verifying:', e);
            return false;
        }
    } catch (e) {
        console.error('Error in signature validation:', e);
        return false;
    }
}

export const handleValidationAndStore = onCall(
    { region: 'us-central1', enforceAppCheck: false },
    async (request: CallableRequest<HeartbeatData>) => {
        console.log('Received data:', request.data);
        const data = request.data;
        const { submitter, payload, signature } = data;

        if (!submitterAllowed(submitter)) {
            throw new functions.https.HttpsError(
                'permission-denied',
                'Public key not authorized',
            );
        }

        const db = getFirestore();

        try {
            if (!validateSignature(payload, signature, submitter)) {
                throw new functions.https.HttpsError(
                    'unauthenticated',
                    'Signature validation failed',
                );
            }

            const rateLimitRef = db.collection('publicKeyRateLimits').doc(submitter);
            const newHeartbeatRef = db.collection('heartbeats').doc();

            await db.runTransaction(async (transaction) => {
                const rateLimitDoc = await transaction.get(rateLimitRef);
                const now = Date.now();
                const windowStart = now - WINDOW_SIZE_MS;

                if (rateLimitDoc.exists) {
                    const data = rateLimitDoc.data();
                    const previousTimestamps: number[] = data?.timestamps || [];
                    const currentWindowTimestamps = previousTimestamps.filter(ts => ts > windowStart);

                    currentWindowTimestamps.push(now);

                    if (currentWindowTimestamps.length > MAX_REQUESTS_PER_WINDOW) {
                        throw new functions.https.HttpsError(
                            'resource-exhausted',
                            'Rate limit exceeded',
                        );
                    }

                    transaction.set(rateLimitRef, {
                        timestamps: currentWindowTimestamps,
                        lastCall: FieldValue.serverTimestamp(),
                    });
                } else {
                    // First request for this public key
                    transaction.set(rateLimitRef, {
                        timestamps: [now],
                        lastCall: FieldValue.serverTimestamp(),
                    });
                }

                transaction.create(newHeartbeatRef, {
                    ...data,
                    createTime: FieldValue.serverTimestamp(),
                });
            });

            return { message: 'Data validated and stored successfully' };
        } catch (error) {
            console.error('Error during data validation and storage:', error);
            if (error instanceof functions.https.HttpsError) {
                throw error;
            }
            throw new functions.https.HttpsError(
                'internal',
                'An error occurred during validation or storage',
            );
        }
    },
);

export { validateSignature };
