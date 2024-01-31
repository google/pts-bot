#!/usr/bin/env python

import argparse
import json

from natsort import natsorted

from pathlib import Path


def sort_ics_in_json(json_file: Path):
    with json_file.open("r") as f:
        json_data = json.load(f)
        ics = json_data["ics"]
        sorted_ics_items = natsorted(ics.items(), key=lambda item: item[0])
        sorted_ics = dict(sorted_ics_items)
        json_data["ics"] = sorted_ics

        return json_data


def main(input_file: Path, output_file: Path):
    sorted_json_data = sort_ics_in_json(input_file)

    with output_file.open("w") as f:
        json.dump(sorted_json_data, f, indent=2)
        print("success")


if __name__ == "__main__":
    """
    Sort the ICS of json pts config file.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--input_file",
        type=Path,
        required=True,
        help="Input Json file to be sorted",
    )
    parser.add_argument(
        "--output_file", type=Path, required=True, help="Output Json file sorted"
    )

    args = parser.parse_args()
    main(**vars(parser.parse_args()))
