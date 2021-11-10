Project: /blueberry/_project.yaml
Book: /blueberry/_book.yaml

# User guide

This user guide explains how to install PTS-bot and Rootcanal and how to build
and test the AOSP Bluetooth Host with it. While not limited to this, PTS-bot,
Rootcanal and the AOSP Bluetooth Host run on the same machine in this guide.

## Requirements

You must have approximately 1GB available on your disk to install PTS-bot and
Rootcanal and 150 GB for the AOSP Bluetooth Host.

## Add Blueberry gLinux repository

PTS-bot and its components are provided as binaries from the Blueberry gLinux
repository, which you should add:

```shell
sudo glinux-add-repo blueberry
sudo apt update
```

## Install Rootcanal

```shell
sudo apt install root-canal
```

## Install PTS-bot

```shell
sudo apt install pts-bot
```

PTS-bot is looking for an official PTS binary to be available in a
`~/.config/pts` folder. Copy it from our shared repository:

```shell
mkdir -p ~/.config/pts
cp /google/data/ro/teams/blueberry/pts_setup_8_0_3.exe ~/.config/pts
```

## Download and build the AOSP Bluetooth Host

1. Make sure you have `repo` installed on your gLinux machine. If you don't,
   install it: `sudo apt install repo`.

1. Create a new folder and download the code of the AOSP Bluetooth Host. As we
   don't need the entire git history here, we use the `--depth 0` option. When
   syncing the repository, you can change the `-jX` option based on the number
   of cores available on your gLinux machine.

   ```shell
   mkdir aosp
   cd aosp
   repo init -u https://android.googlesource.com/platform/manifest -b master --depth 0
   repo sync -j42
   ```

1. Build the AOSP Bluetooth Host stack and Topshim. First, we must check out on
   a specific CL as all the changes have not yet been merged to the AOSP main
   branch. Make sure you are using bash for the following commands.

   ```bash
   source build/envsetup.sh
   lunch 1
   git -C packages/modules/Bluetooth fetch https://android.googlesource.com/platform/packages/modules/Bluetooth refs/changes/82/1882482/1 && git -C packages/modules/Bluetooth checkout FETCH_HEAD
   m bt_topshim_facade
   ```

## Usage

1. Open 3 terminal windows as we must run Rootcanal, PTS-bot and the AOSP
   Bluetooth Host in parallel. Although the three components run on the same
   machine here, they could be run on separate ones as they communicate using
   HTTP and TCP.

1. On the first window, run Rootcanal: `root-canal`. Rootcanal uses TCP on ports
   6401, 6402, 6403 and 6404 for communicating with PTS-bot and the AOSP
   Bluetooth Host.

1. On the second window, run the AOSP Bluetooth Host. As specified in the
   command, it exposes the gRPC test server and its interfaces on port 8999.

   ```bash
   cd aosp
   source build/envsetup.sh
   lunch 1
   $ANDROID_BUILD_TOP/out/host/linux-x86/bin/bt_topshim_facade --blueberry=true --grpc-port=8999
   ```

1. On the third window, run PTS-bot: `pts-bot A2DP/SNK`. As specified in the
   command, it only runs the audio sink tests for the A2DP layer (we still have
   some issues with the source tests that will be fixed soon).

![PTS-bot AOSP architecture](
/blueberry/guides/pts-bot/images/pts-bot-architecture-aosp.svg)

## Going further

* Browsing PTS-bot source code: [`PTS-bot`](
* https://blueberry.git.corp.google.com/PTS-bot/), [`libpts`](
  https://blueberry.git.corp.google.com/libpts/), [`mmi2grpc`](
  https://blueberry.git.corp.google.com/mmi2grpc/).
* Contribute to the [Blueberry test interfaces](
  https://blueberry.git.corp.google.com/bt-test-interfaces/)
