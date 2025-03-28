# Understand PTS tests

PTS Test Cases for a Bluetooth profile/protocol are described in its
corresponding Test Suite. The list of PTS tests applicable to a particular
implementation depends on its Implementation Conformance Statement.

A complete description of the Bluetooth protocol testing strategy is provided
on the [Bluetooth SIG website](
https://www.bluetooth.com/specifications/qualification-test-requirements/), in
the *Other Documents and Templates* section (Test Strategy and Terminology
Overview (Part 1 Vol A)).

## Implementation Conformance Statement (ICS)

Each Bluetooth implementation must provide a statement of the capabilities and
options that have been implemented for each specification that it supports, so
that it can be tested against relevant requirements (and against those
requirements only). This statement is called an Implementation Conformance
Statement (ICS) and is provided in the form of a questionnaire completed by the
implementer.

For example, the latest ICS for the Fluoride stack can be found [here](
https://launchstudio.bluetooth.com/ListingDetails/13841).

A new ICS can be computed for a profile being implemented using
[Launch Studio][launch-studio]. This requires a Bluetooth SIG account.

## Test Suite (TS)

The Bluetooth Test Suite (TS) is the document that notably contains the Test
Strategy, the Test Cases (TCs) and the Test Case Mapping Table (TCMT, mapping
the ICS with their associated TCs) needed to test a specific adopted Bluetooth
specification.

The Test Suite for a Bluetooth profile/protocol can be found on the
[Bluetooth SIG website](https://www.bluetooth.com/specifications/specs/) next to
its specification (qualification test documents tab).

### Test Strategy

In Bluetooth testing, only the observable behavior of the implementation is
tested; the interactions of an implementation with its environment; no
references are made to the internal structure of the protocol implementation
(only black-box testing is performed).

The Bluetooth Test Suites define the roles of the Lower Tester (LT) and Upper
Tester (UT) and the role of the Implementation Under Test (IUT), where
conclusions about the conformance of an IUT are drawn from observing and
controlling the events that occur at the lower and upper service interfaces of
the IUT. Any IUT has to be controllable and observable.

* The **Lower Tester (LT)** interacts with the IUT over-the-air interface. In
  order to do that, it needs to implement the Radio Controller and the parts of
  the Host needed to execute the Lower Tester test steps defined in the Test
  Suite.

* The **Upper Tester (UT)** interacts with the IUT at the upper edge. The upper
  edge interaction may be necessary to initiate certain actions on the IUT, to
  set the IUT in certain states, or to verify data collected on the IUT.

![Lower and upper testers](
/pandora/guides/pts-bot/images/lower-upper-testers.svg){: width="70%"}

When using the standard PTS, the lower tester is the combination of the PTS
software and dongle, and the upper tester is generally the UI of the DUT. A
human operator run the tests by using the PTS software and triggering actions
on the DUT when the PTS requests them through its Man Machine Interfaces (MMIs).

![Lower and upper testers for PTS](
/pandora/guides/pts-bot/images/lower-upper-testers-pts.svg){: width="70%"}

When using PTS-bot, the upper tester is the Pandora Bluetooth test server
implementing the [Pandora APIs](
https://github.com/google/bt-test-interfaces/blob/main/doc/overview.md).
PTS tests are automated by converting the PTS
MMI calls to gRPC calls to the test interfaces.

![Lower and upper testers for PTS-bot](
/pandora/guides/pts-bot/images/lower-upper-testers-pts-bot.svg){: width="80%"}

### Test Case categorization

Each PTS Test Case is identified as either an interoperability Test Case or a
conformance Test Case. These are also referred to as "-I" and "-C" tests.
Interoperability Test Cases refer to non-protocol-related tests or tests that
verify end-to-end system capabilities. Conformance Test Cases refer to all
protocol-related tests.

Each PTS Test Case is also identified whether it tests:

* **A Valid Behavior (BV)**: intends to verify expected behavior as specified.

* **An Invalid Behavior (BI)**: sometimes also referred to as
  "Negative Testing". In this testing, the IUT is exposed to values outside
  defined ranges, or to a Lower Tester that mimics protocol or profile
  behavioral aspects considered invalid or unexpected. The pass criteria in BI
  tests often require the IUT to generate applicable error responses, ignore
  attempted illegal operation, or disconnect from the Lower Tester.

### Test Case Mapping Table (TCMT)

The Test Case Mapping Table (TCMT) maps Test Cases to specific capabilities in
the Implementation Conformance Statement (ICS).

The columns for the TCMT are defined as follows:

* **Item**: this is a logical expression based on specific entries from the
  associated ICS document, using the operators AND, OR, and NOT to determine
  Test Case applicability. The convention is ABRV TABLE/ROW, where ABRV is the
  abbreviated specification name. For example, "ABRV 1/2" refers to
  "Table 1/Row 2" in the corresponding ABRV ICS.

* **Feature**: a brief, informal description of the feature being tested.

* **Test Case(s)**: one or more Test Case ID(s) to be exercised based on the
  supported feature. One line may contain multiple Test Cases, but each Test
  Case appears **once and only once** in the table.

## Abstract Test Suite (ATS)

An Abstract Test Suite (ATS) is defined by an implementation-independent set of
Test Cases.

The PTS ATS for each profile/protocol can be found downloaded from the
certification test tool and provides the list of Test Cases supported by
the PTS. They also provide a list (often not complete) of their associated MMI.

## Implementation Extra Information for Test (IXIT)

They typically contains information about the physical setup and connection of
the test that is not part of the protocol or profile. The IXIT (if any) of a
Bluetooth profile/protocol can be found on the [Bluetooth SIG website](
https://www.bluetooth.com/specifications/specs/) next to their specification
(qualification test documents tab).

[launch-studio]: https://www.bluetooth.com/develop-with-bluetooth/build/test-tools/launch-studio/
