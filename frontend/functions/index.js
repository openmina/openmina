/**
 * Import function triggers from their respective submodules:
 *
 * const {onCall} = require("firebase-functions/v2/https");
 * const {onDocumentWritten} = require("firebase-functions/v2/firestore");
 *
 * See a full list of supported triggers at https://firebase.google.com/docs/functions
 */
//
// const {onRequest} = require('firebase-functions/v2/https');
// const logger = require('firebase-functions/logger');
//
// Create and deploy your first functions
// https://firebase.google.com/docs/functions/get-started
//
// exports.helloWorld = onRequest((request, response) => {
//   logger.info('Hello logs!', {structuredData: true});
//   response.send('Hello from Firebase!');
// });

const admin = require('firebase-admin');
const functions = require('firebase-functions');

admin.initializeApp();

const allowedPublicKeys = new Set([
  'publicKey1Base64Encoded',
  'publicKey2Base64Encoded',
  // Add more as needed
]);

function validateSignature(data, signature, publicKeyBase64) {

  return true;
}

exports.handleValidationAndStore = functions.region('us-central1').https.onCall(async (data, context) => {
  console.log('Received data:', data);
  const {publicKey, data: inputData, signature} = data;

  // Check if the publicKey is in the allowed set
  if (!allowedPublicKeys.has(publicKey)) {
    throw new functions.https.HttpsError('permission-denied', 'Public key not authorized');
  }

  // Rate limiting based on public key
  const rateLimitRef = admin.firestore().collection('publicKeyRateLimits').doc(publicKey);

  try {
    await admin.firestore().runTransaction(async (transaction) => {
      const doc = await transaction.get(rateLimitRef);
      const now = Date.now();
      const cutoff = now - 15 * 1000; // 15 seconds ago

      if (doc.exists) {
        const lastCall = doc.data().lastCall;
        if (lastCall > cutoff) {
          throw new functions.https.HttpsError('resource-exhausted', 'Rate limit exceeded for this public key');
        }
      }

      transaction.set(rateLimitRef, {lastCall: now}, {merge: true});
    });

    // Validate signature
    if (!validateSignature(inputData, signature, publicKey)) {
      throw new functions.https.HttpsError('unauthenticated', 'Signature validation failed');
    }

    // Store data in 'heartbeat' collection
    await admin.firestore().collection('heartbeat').add(inputData);

    return {message: 'Data validated and stored successfully'};
  } catch (error) {
    console.error('Error during data validation and storage:', error);
    if (error instanceof functions.https.HttpsError) {
      throw error; // Re-throw HttpsError for Firebase Functions to handle
    }
    throw new functions.https.HttpsError('internal', 'An error occurred during validation or storage');
  }
});
