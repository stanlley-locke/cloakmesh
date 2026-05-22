# CloakMesh: CloakBrowser Audit & Build Guide

This document provides the commands to build and audit the dedicated privacy browser for the CloakMesh network.

---

## 1. Audit Configuration (Gecko Layer)

Before building, verify that the network lockdown is correctly implemented.

### Network Routing & Telemetry
```bash
# Ensure proxy is hardcoded to 127.0.0.1:9050
cat ../cloak-browser/browser/app/profile/05-custom-network.js | grep "network.proxy.socks"
```

### Custom TLD Hook (C++)
```bash
# Verify .cloak and .onion are registered as valid TLDs
cat ../cloak-browser/netwerk/dns/nsEffectiveTLDService.cpp | grep "EndsWith(\".cloak\")"
```

---

## 2. Build Sequence

The build process requires the `mach` build system and the `.mozconfig` manifest I have provided.

```bash
cd ../cloak-browser

# 1. Initialize the build environment (Select '1. Firefox for Desktop')
./mach bootstrap

# 2. Start the optimized release compilation (Take 30m - 2h)
./mach build
```

---

## 3. Verification & Execution

Run the freshly compiled browser to test the proxy bridge.

```bash
# Launch the browser
./mach run
```

**Manual Test Procedure:**
1.  Open the browser.
2.  Navigate to `about:config`.
3.  Search for `network.proxy.socks`. Verify it is `127.0.0.1`.
4.  Search for `network.proxy.socks_remote_dns`. Verify it is `true`.
5.  Try visiting a hosted site: `http://ahqw6zrrljnem7gxqlducifffw2v7nhgyqujcy36jlfwr5xbmaxfg3iuxdoa.cloak/`
    *   *Result:* The page should load via the local node's onion circuit.

---

## 4. Distribution Packaging

Bundle the browser for end-users:
```bash
./mach package
# Binaries will be in: obj-cloak-browser/dist/
```

"Privacy isn't a feature. It's a foundation."
