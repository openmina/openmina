const { validateSignature } = require('../index');

describe('validateSignature', () => {
    let signedHeartbeat;

    beforeAll(() => {
        signedHeartbeat = JSON.parse(`{
          "version": 1,
          "payload": "eyJzdGF0dXMiOnsiY2hhaW5faWQiOm51bGwsInRyYW5zaXRpb25fZnJvbnRpZXIiOnsiYmVzdF90aXAiOm51bGwsInN5bmMiOnsidGltZSI6bnVsbCwic3RhdHVzIjoiU3luY2VkIiwicGhhc2UiOiJSdW5uaW5nIiwidGFyZ2V0IjpudWxsfX0sInBlZXJzIjpbXSwic25hcmtfcG9vbCI6eyJ0b3RhbF9qb2JzIjowLCJzbmFya3MiOjB9LCJ0cmFuc2FjdGlvbl9wb29sIjp7InRyYW5zYWN0aW9ucyI6MCwidHJhbnNhY3Rpb25zX2Zvcl9wcm9wYWdhdGlvbiI6MCwidHJhbnNhY3Rpb25fY2FuZGlkYXRlcyI6MH0sImN1cnJlbnRfYmxvY2tfcHJvZHVjdGlvbl9hdHRlbXB0IjpudWxsfSwibm9kZV90aW1lc3RhbXAiOjAsInBlZXJfaWQiOiIyYkVnQnJQVHpMOHdvdjJENEt6MzRXVkxDeFI0dUNhcnNCbUhZWFdLUUE1d3ZCUXpkOUgiLCJsYXN0X3Byb2R1Y2VkX2Jsb2NrIjpudWxsfQ==",
          "submitter": "B62qnLjgW4LAnrxkcdLc7Snb49qx6aP5qsmPsp6ueZN4XPMC621cqGc",
          "signature": {
            "field": "25500978175045040705256298774101531557080530394536110798266178142513301557846",
            "scalar": "27991123709623419396663280967637181749724990269901703962618583375785482061803"
          }
        }`);
    });

    test('should validate correct signature', () => {
        const result = validateSignature(
            signedHeartbeat.payload,
            signedHeartbeat.signature,
            signedHeartbeat.submitter
        );
        expect(result).toBe(true);
    });

    test('should reject invalid signature length', () => {
        const result = validateSignature(
            signedHeartbeat.payload,
            'invalid-signature',
            signedHeartbeat.submitter
        );
        expect(result).toBe(false);
    });

    test('should reject tampered data', () => {
        const tamperedPayload = signedHeartbeat.payload + 'tampered';
        const result = validateSignature(
            tamperedPayload,
            signedHeartbeat.signature,
            signedHeartbeat.submitter
        );
        expect(result).toBe(false);
    });

    test('should handle null values', () => {
        expect(validateSignature(null, null, null)).toBe(false);
        expect(validateSignature(signedHeartbeat.payload, null, signedHeartbeat.submitter)).toBe(false);
        expect(validateSignature(signedHeartbeat.payload, signedHeartbeat.signature, null)).toBe(false);
    });
});
