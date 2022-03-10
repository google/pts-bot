#!/usr/bin/env python3

import argparse
import json
from pathlib import Path
from xml.dom import minidom

def pts2json(pts_file: Path, output: Path):
    TSPC = {'ics': {}, 'ixit': {}}
    ics = TSPC['ics']
    ixit = TSPC['ixit']
    ixit['default'] = {}
    ics_xml = minidom.parse(str(pts_file))
    profiles = ics_xml.getElementsByTagName('profile')
    for profile in profiles:
        profile_name = profile.getElementsByTagName('name')[0].firstChild.nodeValue
        ixit[profile_name] = {}
        items = profile.getElementsByTagName('item')
        for item in items:
            table = item.getElementsByTagName('table')[0].firstChild.nodeValue
            row = item.getElementsByTagName('row')[0].firstChild.nodeValue
            tspc = f"TSPC_{profile_name}_{table}_{row}"
            ics[tspc] = True
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
