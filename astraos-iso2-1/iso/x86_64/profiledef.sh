#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════════════╗
# ║              AstraOS ISO 2 — Profile Definition Script               ║
# ║     Configures the settings and parameters for the AstraOS ISO 2     ║
# ║            © 2026 AstraOS Project — Astra Corporation                ║
# ╚══════════════════════════════════════════════════════════════════════╝

# Define the profile name
PROFILE_NAME="AstraOS ISO 2"

# Define the version
VERSION="2.0"

# Define the architecture
ARCHITECTURE="x86_64"

# Define the description
DESCRIPTION="AstraOS ISO 2 - A lightweight and user-friendly operating system."

# Define the packages to be included in the ISO
PACKAGES=(
    base
    linux
    linux-firmware
    vim
    networkmanager
    # Add other necessary packages here
)

# Function to display profile information
display_profile_info() {
    echo "Profile Name: ${PROFILE_NAME}"
    echo "Version: ${VERSION}"
    echo "Architecture: ${ARCHITECTURE}"
    echo "Description: ${DESCRIPTION}"
    echo "Packages: ${PACKAGES[*]}"
}

# Call the function to display profile information
display_profile_info