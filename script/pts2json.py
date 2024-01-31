#!/usr/bin/env python3

import argparse
import json

from natsort import natsorted
from pathlib import Path
from typing import Dict
from xml.dom import minidom

def sort_ics(ics: Dict[str, bool]) -> Dict[str, bool]:
    sorted_ics_items = natsorted(ics.items(), key=lambda item: item[0])
    sorted_ics = dict(sorted_ics_items)

    return sorted_ics

def pts2json(pts_file: Path, output: Path):
    TSPC = {'ics': {}, 'ixit': {}}
    ics = TSPC['ics']
    ixit = TSPC['ixit']
    ixit['default'] = {}
    ics_xml = minidom.parse(str(pts_file))
    profiles = ics_xml.getElementsByTagName('profile')
    for profile in profiles:
        profile_name = profile.getElementsByTagName('name')[0].firstChild.nodeValue  # type: ignore
        ixit[profile_name] = {}
        items = profile.getElementsByTagName('item')
        for item in items:
            table = item.getElementsByTagName('table')[0].firstChild.nodeValue  # type: ignore
            row = item.getElementsByTagName('row')[0].firstChild.nodeValue  # type: ignore
            tspc = f"TSPC_{profile_name}_{table}_{row}"
            ics[tspc] = True
    TSPC['ics'] = sort_ics(ics)
    with open(output, "w") as config_file:
        json.dump(TSPC, config_file, indent=2)
        print(f'success: {output}')

def main():
    parser = argparse.ArgumentParser(description="Parse pts file to json")
    parser.add_argument('--pts_file',
                        type=Path,
                        required=True,
                        help='pts config file generated with Launch Studio')
    parser.add_argument('--output',
                        type=Path,
                        required=False,
                        default=Path('pts_config.json'),
                        help='Output name for the pts config file')
    pts2json(**vars(parser.parse_args()))

if __name__ == "__main__":
  main()
