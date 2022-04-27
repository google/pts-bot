Project: /pandora/_project.yaml
Book: /pandora/_book.yaml

# PTS-bot

**PTS-bot** is an automation of the Bluetooth Profile Tuning Suite (PTS), which
is the testing tool provided by the Bluetooth standard to run Host certification
tests. PTS-bot leverages the tests provided by the PTS but removes the need for
a human operator and for a physical Bluetooth dongle to run them.

![PTS-bot overview](
/pandora/guides/pts-bot/images/pts-bot.svg){: width="80%"}

## What is the PTS?

The [Bluetooth Profile Tuning Suite (PTS)](
https://www.bluetooth.com/develop-with-bluetooth/qualification-listing/qualification-test-tools/profile-tuning-suite/)
is a Windows testing software that automates certification testing to the
specified functional requirements of Bluetooth Host Parts (i.e. specifications
that reside above the Host Controller Interface (HCI)).

As stated by the Bluetooth SIG, the vision of the PTS is to provide complete and
validated test coverage, of all specified functional requirements in scope, to
the Bluetooth  development and testing community.

However, the PTS currently has three major limitations, which makes it
impossible to automate and hard to use:

* It only runs on Windows.

* It requires a Bluetooth USB dongle to work, and thus relies on a physical
  communication to the DUT, often requiring to rerun tests due to a poor
  reliability.

* It requires a human operator (notably by using popups for manual actions).

## Introducing PTS-bot

PTS-bot fixes the three major limitations of the PTS:

* **It runs the official PTS binary in [Wine](https://www.winehq.org/)**, a
  compatibility layer allowing to run Windows applications on POSIX OS such as
  Linux.

* **It can emulate the Bluetooth communication between the PTS and the DUT using
  Rootcanal**, a virtual Bluetooth Controller, initialy built for AOSP, removing
  the need for a physical communication. HCI calls on the DUT are routed to
  Rootcanal instead of the Bluetooth chip.

* **It automates commands to the DUT through Bluetooth test interfaces (gRPC)**
  exposed by each layer of the Bluetooth stack. A translation layer is built to
  convert the actions on the DUT requested by the PTS through its Man Machine
  Interfaces (MMIs) to gRPC, removing the need for a human operator.

### Goals

PTS-bot has been built as an important first piece of Pandora:

* **It provides a huge number of tests, without the need to implement the
  entire test framework**, which addresses the urgency of having tests for
  Google Bluetooth stacks.

* **It will allow defining and unifying all Bluetooth test interfaces of
  Pandora**, both low-level and high-level (as PTS tests cover both). Other
  types of Pandora tests (device-to-device or interoperability) will then use
  the same interfaces.

* **It will allow pre-certifying DUTs** using a virtual Bluetooth communication
  without all the issues that the PTS usually have when running physically,
  allowing it to be run in a fast and repeatable fashion. It can also enable
  automation of the physical PTS tests.

PTS-bot aims to be used by Bluetooth stack developers, locally on their machines
and within pre-submit tests, to verify that their code is passing the minimum
Bluetooth test requirements to avoid introducing regressions.

### Architecture

PTS-bot is relying on three components:

* [`libpts`](https://blueberry.git.corp.google.com/libpts/) manages the PTS
  environment, including the Wine server for running the PTS Windows binary and
  the PTS parser, used to produce well structured logs and to parse the PTS MMI.
  This library is mostly written in Rust.

* [`mmi2grpc`](https://pandora.git.corp.google.com/mmi2grpc/) acts as a gRPC
  client (the gRPC server being implemented on the DUT) and translates PTS MMIs
  into gRPC calls. This library is written in python so as to be easily updated
  by other developers. It includes the Bluetooth gRPC test interfaces generated
  from their protobuf definitions located in the [`bt-test-interfaces`](
  https://pandora.git.corp.google.com/bt-test-interfaces/) repository. Those
  interfaces are not solely designed for PTS-bot but aim to be used for all
  tests interacting with a Google Bluetooth stack (for both Android and embedded
  devices).

* **Rootcanal**, a virtual Bluetooth Controller used to simulate the Bluetooth
  communication.

![PTS-bot architecture](
/pandora/guides/pts-bot/images/pts-bot-architecture.svg){: width="90%"}

PTS-bot can run on the same machine as the Bluetooth stack to be tested and/or
Rootcanal (for instance all being run on the same Linux computer, or within the
same Android virtual device) or on separate machines as Rootcanal uses TCP
and gRPC uses HTTP.

### Limitations

PTS-bot has two limitations:

* Because it relies on the PTS binary for providing tests, it is limited to the
  same test coverage, and cannot be extended by custom tests. This means that
  passing PTS-bot tests is necessary but not sufficient (as some
  interoperability issues are not covered by the PTS).

* Because it relies on a virtual Bluetooth Controller, it cannot check for any
  potential issues located inside the Bluetooth chip of a specific device. This
  means again that PTS-bot must be supplemented by physical tests.

### Going further

* Browsing PTS-bot source code: [`PTS-bot`](
  https://blueberry.git.corp.google.com/PTS-bot/), [`libpts`](
  https://blueberry.git.corp.google.com/libpts/), [`mmi2grpc`](
  https://pandora.git.corp.google.com/mmi2grpc/).
* Contribute to the [Bluetooth test interfaces](
  https://pandora.git.corp.google.com/bt-test-interfaces/)
