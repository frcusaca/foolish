#!/bin/bash

# Configuration
SOURCE_REPO="$HOME/.m2/repository"
TARGET_REPO="$(pwd)/.m2.4.me"

rm -rf ${SOURCE_REPO}/org/foolish

# Handle --force or -f flags
if [[ "$1" == "-L" ]]; then
    echo "Lazy model, don't delete existing $TARGET_REPO..."
else
    rm -rf "$TARGET_REPO"
fi

if [ ! -d "$SOURCE_REPO" ]; then
    echo "Error: Source repository $SOURCE_REPO does not exist."
    exit 1
fi

if [ -e "$TARGET_REPO" ]; then
    echo "Target $TARGET_REPO already exists. Skipping copy."
else
    OS_TYPE=$(uname -s)
    echo "Detected OS: $OS_TYPE"

    case "$OS_TYPE" in
        Darwin)
            # macOS (APFS) creates COW clones automatically with -R
            echo "Creating APFS clone..."
            cp -R "$SOURCE_REPO" "$TARGET_REPO"
            ;;
        Linux)
            # Attempt reflink (Btrfs, XFS, ZFS 2.2+)
            echo "Attempting reflink copy..."
            if cp --reflink=always -r "$SOURCE_REPO" "$TARGET_REPO" 2>/dev/null; then
                echo "Success: Created space-efficient reflink copy."
            else
                echo "Reflink failed (unsupported filesystem). Falling back to standard shared copy..."
    		rm -rf "$TARGET_REPO" # clean up the failed copy
                #cp -r "$SOURCE_REPO" "$TARGET_REPO"
                ln -s "$SOURCE_REPO" "$TARGET_REPO"
            fi
            ;;
        *)
            echo "Unknown OS. Performing standard shared copy..."
            ln -s "$SOURCE_REPO" "$TARGET_REPO"
            ;;
    esac
fi

# Export for the current shell session
# Using $(realpath ...) ensures Maven always has an absolute path
export MAVEN_OPTS="-Dmaven.repo.local=$TARGET_REPO"

echo "----------------------------------------------------------"
echo "Local Repo: $TARGET_REPO"
echo "MAVEN_OPTS set. Ready for: mvn compile / install"
echo "Run following in all sessions that invokes maven"
echo export MAVEN_OPTS=\"-Dmaven.repo.local=$TARGET_REPO\"
echo "----------------------------------------------------------"

