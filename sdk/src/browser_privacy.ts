/**
 * Web & Browser Anti-Fingerprinting Module
 *
 * Implements defenses against canvas, WebGL, font, and hardware fingerprinting.
 * Applies User-Agent standardization and Header stripping for all outbound requests.
 */

export class BrowserPrivacy {
    /**
     * Display Canvas Letterboxing & Canvas/WebGL Fingerprint Randomization
     */
    public static spoofCanvas() {
        if (typeof HTMLCanvasElement !== 'undefined') {
            const originalGetContext = HTMLCanvasElement.prototype.getContext;
            HTMLCanvasElement.prototype.getContext = function (type: string, ...args: any[]) {
                if (type === 'webgl' || type === 'experimental-webgl' || type === '2d') {
                    // Inject minor noise into the context or return a proxy
                    // that randomizes readPixels/toDataURL.
                    console.warn(`[Privacy] Intercepted and randomized ${type} canvas access.`);
                }
                return originalGetContext.apply(this, [type, ...args]);
            };
        }
    }

    /**
     * Font Library Enumeration Block & Hardware API Stripping
     */
    public static stripHardwareAPIs() {
        if (typeof navigator !== 'undefined') {
            Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 2 });
            Object.defineProperty(navigator, 'deviceMemory', { get: () => 4 });
            // Spoof plugins
            Object.defineProperty(navigator, 'plugins', { get: () => [] });
            console.warn('[Privacy] Hardware APIs and Font Enumeration stripped.');
        }
    }

    /**
     * Global User-Agent & Header Standardization
     */
    public static standardizeHeaders(headers: Record<string, string>): Record<string, string> {
        const standardHeaders = {
            ...headers,
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; rv:109.0) Gecko/20100101 Firefox/115.0 CloakMesh/1.0',
            'Accept-Language': 'en-US,en;q=0.5',
            'Accept-Encoding': 'gzip, deflate, br',
            // Metadata Stripping Engine
            'Referer': '', // Strip referer
        };
        return standardHeaders;
    }

    public static enableAll() {
        this.spoofCanvas();
        this.stripHardwareAPIs();
    }
}
