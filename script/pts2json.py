#!/usr/bin/env python3

import sys
import json
from xml.dom import minidom

_file_name = "config.json"

def to_json(ics_xml_path: str):
    global _file_name
    if len(sys.argv) > 2:
        _file_name = sys.argv[2]
    config_file = open(_file_name, "w")
    TSPC = {'ics': {}, 'ixit': {}}
    ics = TSPC['ics']
    ics_xml = minidom.parse(ics_xml_path)
    profiles = ics_xml.getElementsByTagName('profile')
    for profile in profiles:
        profile_name = profile.getElementsByTagName('name')[0].firstChild.nodeValue
        items = profile.getElementsByTagName('item')
        for item in items:
            table = item.getElementsByTagName('table')[0].firstChild.nodeValue
            row = item.getElementsByTagName('row')[0].firstChild.nodeValue
            tspc = f"TSPC_{profile_name}_{table}_{row}"
            ics[tspc] = True
    json.dump(TSPC, config_file, indent=2)
    print(f'success: {_file_name}')

def main():
    if len(sys.argv) > 1:
        print(f'pts file: {sys.argv[1]}')
        to_json(sys.argv[1])
    else:
        print("pts file needed as argument")

if __name__ == "__main__":
  main()
