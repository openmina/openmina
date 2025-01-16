import * as admin from 'firebase-admin';
import * as functions from 'firebase-functions';
import * as blake2 from 'blake2';
import bs58check from 'bs58check';
import Client from 'mina-signer';
import { submitterAllowed } from './submitterValidator';

interface SignatureJson {
    field: string;
    scalar: string;
}

interface HeartbeatData {
    publicKey: string;
    data: string;
    signature: SignatureJson;
}

const minaClient = new Client({ network: 'testnet' });

admin.initializeApp();

function validateSignature(
    data: string,
    signature: SignatureJson,
    publicKeyBase58: string
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

export const handleValidationAndStore = functions
    .region('us-central1')
    .https.onCall(async (data: HeartbeatData, context: functions.https.CallableContext) => {
        console.log('Received data:', data);
        const { publicKey, data: inputData, signature } = data;

        if (!submitterAllowed(publicKey)) {
            throw new functions.https.HttpsError(
                'permission-denied',
                'Public key not authorized'
            );
        }

        const rateLimitRef = admin.firestore().collection('publicKeyRateLimits').doc(publicKey);

        try {
            await admin.firestore().runTransaction(async (transaction) => {
                const doc = await transaction.get(rateLimitRef);
                const now = Date.now();
                const cutoff = now - 15 * 1000;

                if (doc.exists) {
                    const lastCall = doc.data()?.lastCall;
                    if (lastCall > cutoff) {
                        throw new functions.https.HttpsError(
                            'resource-exhausted',
                            'Rate limit exceeded for this public key'
                        );
                    }
                }

                transaction.set(rateLimitRef, { lastCall: now }, { merge: true });
            });

            if (!validateSignature(inputData, signature, publicKey)) {
                throw new functions.https.HttpsError(
                    'unauthenticated',
                    'Signature validation failed'
                );
            }

            await admin.firestore().collection('heartbeat').add(data);

            return { message: 'Data validated and stored successfully' };
        } catch (error) {
            console.error('Error during data validation and storage:', error);
            if (error instanceof functions.https.HttpsError) {
                throw error;
            }
            throw new functions.https.HttpsError(
                'internal',
                'An error occurred during validation or storage'
            );
        }
    });

export { validateSignature };
