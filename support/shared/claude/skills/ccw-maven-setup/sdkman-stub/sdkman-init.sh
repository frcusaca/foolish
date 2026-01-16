#!/bin/bash
# Stub SDKMAN init script for CCW environments where SDKMAN cannot be installed
# This provides minimal sdk command functionality using system Java

# Define sdk command stub
sdk() {
    local command="$1"
    shift

    case "$command" in
        "list")
            if [ "$1" = "java" ]; then
                # Report the system Java version as if it's installed via SDKMAN
                local java_version=$(java -version 2>&1 | head -1 | awk -F'"' '{print $2}')
                echo "Available Java Versions"
                echo "================================================================================"
                if [[ "$java_version" == 21.* ]]; then
                    echo " * 21-tem                                                            installed"
                elif [[ "$java_version" == 25.* ]]; then
                    echo " * 25-tem                                                            installed"
                else
                    echo " * $java_version                                                     installed"
                fi
                echo "================================================================================"
            fi
            ;;
        "install")
            echo "Note: Using system Java instead of installing via SDKMAN"
            echo "System Java version: $(java -version 2>&1 | head -1)"
            ;;
        "use")
            echo "Note: Using system Java $(java -version 2>&1 | head -1 | awk -F'"' '{print $2}')"
            ;;
        "version")
            echo "SDKMAN stub v1.0.0 (local fallback for CCW)"
            ;;
        *)
            echo "SDKMAN stub: command '$command' not implemented"
            return 1
            ;;
    esac
}

export -f sdk
echo "SDKMAN stub initialized (using system Java)"
