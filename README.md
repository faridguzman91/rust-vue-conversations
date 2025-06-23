## rust-vue-conversations

This project is a full-stack VoIP and messaging platform using:

Rust for backend API and S3 integration

Vue for the frontend UI

Kamailio as the SIP signaling server

FreeSWITCH as the media/voicemail server


## Quick Start
Frontend (Vue)

```
cd frontend-conversations
npm install
npm run dev
```


## Backend (Rust)
```
cd backend-conversations
cargo run

```

## SIP Server (Kamailio)
Install Kamailio
```
git clone https://github.com/kamailio/kamailio.git kamailio
cd kamailio
git checkout -b 5.3 origin/5.3
sudo make cfg
sudo make all
sudo make install

```

# Run Kamailio
```
kamailio -f /your-path-to-kamailio/kamailio.cfg -DD
```


# Media Server (FreeSWITCH)
Install All Required Dependencies


```
sudo apt-get update
sudo apt-get install -y \
  build-essential git pkg-config autoconf automake libtool libtool-bin \
  cmake uuid-dev libssl-dev libcurl4-openssl-dev libpcre3-dev \
  libspeexdsp-dev libedit-dev libldns-dev libopus-dev libsndfile1-dev \
  libavformat-dev libavcodec-dev libswscale-dev \
  libpq-dev libmariadb-dev libspandsp-dev libsofia-sip-ua-dev \
  libsqlite3-dev yasm nasm

```


# Install libks2 (Required for mod_verto and SignalWire modules)
```
cd /usr/local/src
sudo git clone https://github.com/signalwire/libks.git
cd libks
sudo cmake .
sudo make
sudo make install
sudo ldconfig
```
# Install signalwire-c (Required for mod_signalwire)


```
cd /usr/local/src
sudo git clone https://github.com/signalwire/signalwire-c.git
cd signalwire-c
sudo cmake .
sudo make
sudo make install
sudo ldconfig
```
# Build and Install FreeSWITCH
```
git clone https://github.com/signalwire/freeswitch.git
cd freeswitch
sudo ./bootstrap.sh -j
sudo ./configure
sudo make
sudo make install
sudo make all cd-sounds-install cd-moh-install

```


# Troubleshooting
If you see errors about missing packages during ./configure, install the required -dev package and re-run ./configure.

For assembler errors (libvpx):
```sudo apt-get install yasm nasm```

For uuid errors:
``sudo apt-get install uuid-dev``

For spandsp errors:
``sudo apt-get install libspandsp-dev``

For sofia-sip errors:
``sudo apt-get install libsofia-sip-ua-dev``

For sqlite3 errors:
``sudo apt-get install libsqlite3-dev``
```
```
