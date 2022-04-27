Project: /pandora/_project.yaml
Book: /pandora/_book.yaml

# AOSP user guide

This user guide explains how to install, configure, and use PTS-bot on AOSP.

## Prerequisites

You must have approximately 1GB available on your disk to install PTS-bot and
have a recent `master-with-phones` AOSP build.

If you want to run virtual PTS-bot tests on a physical DUT, it also requires
to have Rootcanal installed and running on it (this is done automatically for
Cuttlefish instances).

## How it works

Pandora gRPC Bluetooth test interfaces are implemented on top of the
Android Bluetooth module in the Android Bluetooth test server, which is an
instrumented helper test app (located in
`/packages/modules/Bluetooth/android/pandora/server/`) that exposes a gRPC
server on TCP port 8999. PTS-bot binary is run on the host device and triggers
actions on the DUT through the server.

Tests can be run using a virtual Bluetooth communication (with Rootcanal) or a
physical one. Rootcanal runs on the DUT and also exposes the HCI traffic on a
TCP port (7200 for a Cuttlefish instance, and 6211 for a physical DUT).

The list of PTS tests to execute is computed from a JSON configuration file
which contains the [ICS](/pandora/guides/pts-bot/pts-tests).

The entire process is orchestrated using `atest` and Tradefed to
build/install/run the Android Bluetooth test server, run PTS-bot and retrieve
logs from both.

![PTS-bot on AOSP using Tradefed](
/pandora/guides/pts-bot/images/pts-bot-aosp-tradefed.svg){: width="90%"}

## Usage

Follow the instructions below to install, configure, and run PTS-bot.

### Add Pandora gLinux repository

PTS-bot is provided as a binary package from the Pandora gLinux repository,
which you should add:

```shell
sudo glinux-add-repo blueberry unstable
sudo apt update
```

### Install PTS-bot

```shell
sudo apt install pts-bot
```

PTS-bot is looking for an official PTS binary to be available in a
`~/.config/pts` folder. Copy it from our shared repository:

```shell
mkdir -p ~/.config/pts
cp /google/data/ro/teams/blueberry/pts_setup_8_0_3.exe ~/.config/pts
```

### Clone the `mmi2grpc` repository

```shell
git clone sso://pandora/mmi2grpc
```

### Configure PTS-bot

Modify PTS-bot configuration located in
`/packages/modules/Bluetooth/android/pandora/server/configs/PtsBotTest.xml`
to specify:

* `mmi2grpc`: the path to your local `mmi2grpc` repository.
* `config`: PTS tests JSON configuration file to use. By default, it takes
  PTS-bot AOSP configuration file (which covers all the tests that have already
  been verified).
* `physical`: whether you want to run your tests using a physical communication
  or a virtual one (with Rootcanal). By default, tests are run virtually.
  Reminder: running tests physically may require additional setup.

### Run PTS-bot

```shell
atest pts-bot -v
```

## Running PTS-bot with a physical Bluetooth communication

Running physical PTS-bot tests can be helpful to debug some issues observed in
virtual runs. Since the host device is generally remote (on a Cloudtop instance
for example), running physical tests requires to use an HCI proxy to forward the
Bluetooth dongle HCI traffic to PTS-bot on a dedicated TCP port (1234).

![PTS-bot on a remote host](
/pandora/guides/pts-bot/images/pts-bot-aosp-tradefed-physical.svg){: width="90%"}
