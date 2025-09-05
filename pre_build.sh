#!/bin/sh

echo "Running pre-build script for ALSA dependencies..."

# For musl builds, ALSA often doesn't work well, so skip it
if echo "$RUSTTARGET" | grep -q "musl"; then
    echo "Musl target detected - skipping ALSA installation and using fallback"
    export ALSA_NO_PKG_CONFIG=1
    echo "::info PRE BUILD RAN - ALSA DISABLED FOR MUSL"
    exit 0
fi

# Try different package managers based on what's available
if command -v apk >/dev/null 2>&1; then
    echo "Using apk (Alpine Linux)"
    apk update
    apk add alsa-lib-dev pkgconfig || {
        echo "Failed to install ALSA with apk, falling back to no ALSA"
        export ALSA_NO_PKG_CONFIG=1
    }
elif command -v apt-get >/dev/null 2>&1; then
    echo "Using apt-get (Debian/Ubuntu)"
    apt-get update
    apt-get install -y libasound2-dev pkg-config || {
        echo "Failed to install ALSA with apt-get, falling back to no ALSA"
        export ALSA_NO_PKG_CONFIG=1
    }
elif command -v yum >/dev/null 2>&1; then
    echo "Using yum (RedHat/CentOS)"
    yum install -y alsa-lib-devel pkgconfig || {
        echo "Failed to install ALSA with yum, falling back to no ALSA"
        export ALSA_NO_PKG_CONFIG=1
    }
elif command -v dnf >/dev/null 2>&1; then
    echo "Using dnf (Fedora)"
    dnf install -y alsa-lib-devel pkgconfig || {
        echo "Failed to install ALSA with dnf, falling back to no ALSA"
        export ALSA_NO_PKG_CONFIG=1
    }
else
    echo "No supported package manager found. Attempting to continue without ALSA..."
    echo "Setting ALSA_NO_PKG_CONFIG=1 to bypass ALSA dependency"
    export ALSA_NO_PKG_CONFIG=1
fi

echo "::info PRE BUILD RAN"