// base58 encoded public keys that are allowed to submit data
const allowedPublicKeys: Set<string> = new Set([
    // ALLOWED_PUBLIC_KEYS_PLACEHOLDER
]);

export function submitterAllowed(publicKeyBase58: string): boolean {
    if (allowedPublicKeys.size === 0) {
        return true;
    }
    return allowedPublicKeys.has(publicKeyBase58);
}
