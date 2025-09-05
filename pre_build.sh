#!/bin/sh

echo "Running pre-build script for ALSA dependencies..."

# Try different package managers based on what's available
if command -v apk >/dev/null 2>&1; then
    echo "Using apk (Alpine Linux)"
    apk update
    apk add alsa-lib-dev pkgconfig
elif command -v apt-get >/dev/null 2>&1; then
    echo "Using apt-get (Debian/Ubuntu)"
    apt-get update
    apt-get install -y libasound2-dev pkg-config
elif command -v yum >/dev/null 2>&1; then
    echo "Using yum (RedHat/CentOS)"
    yum install -y alsa-lib-devel pkgconfig
elif command -v dnf >/dev/null 2>&1; then
    echo "Using dnf (Fedora)"
    dnf install -y alsa-lib-devel pkgconfig
else
    echo "No supported package manager found. Attempting to continue without ALSA..."
    echo "Setting ALSA_NO_PKG_CONFIG=1 to bypass ALSA dependency"
    export ALSA_NO_PKG_CONFIG=1
fi

echo "::info PRE BUILD RAN"