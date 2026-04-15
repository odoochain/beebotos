#!/bin/bash
set -e

echo "Setting up TEE (Trusted Execution Environment) for BeeBotOS..."

# Detect TEE type
if [ -d "/dev/sgx" ] || [ -e "/dev/sgx_enclave" ]; then
    TEE_TYPE="SGX"
elif grep -q "SEV" /proc/cpuinfo 2>/dev/null; then
    TEE_TYPE="SEV"
elif [ -d "/sys/firmware/devicetree/base/optee" ]; then
    TEE_TYPE="TrustZone"
else
    echo "No TEE detected. Installing software simulation mode."
    TEE_TYPE="SIM"
fi

echo "Detected TEE type: $TEE_TYPE"

# Install TEE SDK based on type
case $TEE_TYPE in
    SGX)
        echo "Setting up Intel SGX..."
        
        # Install SGX driver and PSW
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            case $ID in
                ubuntu)
                    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx-archive-keyring.gpg] https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main" | \
                        sudo tee /etc/apt/sources.list.d/intel-sgx.list
                    wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | \
                        sudo tee /usr/share/keyrings/intel-sgx-archive-keyring.gpg > /dev/null
                    sudo apt update
                    sudo apt install -y libsgx-urts libsgx-launch libsgx-epid libsgx-quote-ex
                    ;;
                *)
                    echo "Please manually install Intel SGX SDK for your distribution"
                    exit 1
                    ;;
            esac
        fi
        
        # Install Rust SGX SDK
        git clone https://github.com/apache/incubator-teaclave-sgx-sdk.git /opt/sgx-sdk
        
        # Verify SGX
        /opt/intel/sgxsdk/bin/x64/sgx_sign -help > /dev/null && echo "SGX SDK installed successfully"
        ;;
        
    SEV)
        echo "Setting up AMD SEV..."
        
        # Install SEV tools
        sudo apt update
        sudo apt install -y sev-tool
        
        # Build SEV guest tools
        git clone https://github.com/AMDESE/sev-guest.git /tmp/sev-guest
        cd /tmp/sev-guest
        make
        sudo make install
        ;;
        
    TrustZone)
        echo "Setting up OP-TEE..."
        
        # Install OP-TEE dependencies
        sudo apt update
        sudo apt install -y android-tools-adb android-tools-fastboot
        
        # Clone OP-TEE
        mkdir -p /opt/optee
        cd /opt/optee
        repo init -u https://github.com/OP-TEE/manifest.git -m qemu_v8.xml
        repo sync
        ;;
        
    SIM)
        echo "Setting up TEE simulation mode..."
        
        # Install simulation libraries for testing
        cargo install gramine
        
        # Create simulation config
        mkdir -p ~/.beebotos/tee
        cat > ~/.beebotos/tee/config.toml << 'EOF'
[tee]
type = "simulation"
enclave_size = "256M"
thread_num = 4

[attestation]
enabled = false
provider = "simulation"
EOF
        ;;
esac

echo ""
echo "TEE Setup Complete!"
echo "TEE Type: $TEE_TYPE"
echo ""
echo "Next steps:"
echo "1. Review TEE configuration in ~/.beebotos/tee/config.toml"
echo "2. Run TEE tests: cargo test --features tee-$TEE_TYPE"
echo "3. For production, ensure hardware TEE is properly configured"
