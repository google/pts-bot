#!/usr/bin/env python3
#
# Copyright 2025 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import argparse
import dataclasses
import json
import os
import pypdf
import re
import svgwrite
import sys
from dataclasses import dataclass
from typing import List, Optional

# List of MMIs that have incorrect ID in the ATS documents and should be
# patched.
patched_mmis = dict([
    (("ACS", 139, "MMI_WAIT_FOR_PROCEDURE_TIMEOUT"), 140),
    (("ACP", 139, "MMI_WAIT_FOR_PROCEDURE_TIMEOUT"), 140),
    (("BPP", 26, "TSC_MMI_confirm_print"), 27),
])
# List of MMIs that have duplicate ID in the ATS documents and should be
# filtered.
filtered_mmis = set([
    ("AIOP", 49, "MMI_IUT_INITIATE_ACL_CONNECTION"),
    ("AIOS", 49, "MMI_IUT_INITIATE_ACL_CONNECTION"),
    ("BLP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("BLS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("CGMP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("CGMP", 49, "MMI_IUT_INITIATE_ACL_CONNECTION"),
    ("CGMS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("CGMS", 49, "MMI_IUT_INITIATE_ACL_CONNECTION"),
    ("CSIP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("CSIS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("ESLP", 109, "MMI_IUT_VERYFY_IGNORE_MSG"),
    ("ESLP", 110, "MMI_WAIT_FOR_PERIODIC_RESPONSE_PACKET"),
    ("ESLP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("ESLS", 109, "MMI_IUT_VERYFY_IGNORE_MSG"),
    ("ESLS", 110, "MMI_WAIT_FOR_PERIODIC_RESPONSE_PACKET"),
    ("ESLS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("ESP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("ESS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("GLP", 49, "MMI_IUT_INITIATE_ACL_CONNECTION"),
    ("GLS", 49, "MMI_IUT_INITIATE_ACL_CONNECTION"),
    ("HRP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("HRS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("L2CAP", 13, "TSC_MMI_iut_enable_connection"),
    ("L2CAP", 14, "TSC_MMI_iut_disable_connection"),
    ("L2CAP", 15, "TSC_MMI_tester_enable_connection"),
    ("L2CAP", 22, "TSC_A2MP_info_rsp_data_extended_features_mask"),
    ("MCP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("MCS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("MESH", 1000, "MMI_IUT_IN_GENERAL_DISCOVERABLE_MODE"),
    ("MESH", 1001, "MMI_IUT_INITIATE_CONNECTION_API"),
    ("MESH", 2000, "MMI_VERIFY_SECURE_ID"),
    ("MESH", 2001, "MMI_CONFIRM_PASSKEY"),
    ("MMDL", 1000, "MMI_IUT_IN_GENERAL_DISCOVERABLE_MODE"),
    ("MMDL", 1001, "MMI_IUT_INITIATE_CONNECTION_API"),
    ("MMDL", 2000, "MMI_VERIFY_SECURE_ID"),
    ("MMDL", 2001, "MMI_CONFIRM_PASSKEY"),
    ("OTP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("OTS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("PASP", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("PASS", 15, "MMI_TESTER_ENABLE_CONNECTION"),
    ("RCP", 49, "MMI_IUT_INITIATE_ACL_CONNECTION"),
    ("RCS", 49, "MMI_IUT_INITIATE_ACL_CONNECTION"),
])

@dataclass
class Float:
    """Wrapper class for float which approximates comparisons within a margin.
    Necessary since the tables defined in the ATS documents have non exact
    coordinates for the cell edges."""
    f: float
    margin: float = 2.0

    def __lt__(self, other):
        return self.f + self.margin < other.f

    def __le__(self, other):
        return self.f - self.margin < other.f

    def __eq__(self, other):
        return abs(self.f - other.f) < self.margin

    def __add__(self, other):
        return Float(self.f + other.f)

    def __sub__(self, other):
        return Float(self.f - other.f)

    def __repr__(self):
        return f"{self.f}"

@dataclass
class Rect:
    """Represents a rectangle in the ATS document. 0,0 is located at the bottom
    left of the page."""
    x: Float
    y: Float
    w: Float
    h: Float
    text: List['Text'] = dataclasses.field(default_factory=list)

    def __lt__(self, other) -> bool:
        return (self.x, self.y, self.w, self.h) < (other.x, other.y, other.w, other.h)

    def contains(self, x: Float, y: Float) -> bool:
        return (self.x <= x and x < (self.x + self.w) and
                self.y <= y and y < (self.y + self.h))

    def insert(self, text: 'Text'):
        if self.contains(text.x, text.y):
            self.text.append(text)

    def contents(self):
        self.text.sort()
        t = ''.join(t.text for t in self.text)
        return t.replace('\n', '').strip().rstrip()

@dataclass
class VLine:
    x: Float
    y: Float
    l: Float

@dataclass
class HLine:
    x: Float
    y: Float
    l: Float

    def __lt__(self, other) -> bool:
        # Horizontal lines are sorted by y coordinate first to be able to
        # iterate on aligned, adjoined horizontal segments.
        return (self.y, self.x, self.l) < (other.y, other.x, other.l)

@dataclass
class Row:
    """Represents an identified row of cells in any table in the ATS document."""
    cells: List[Rect]

    def insert(self, text: 'Text'):
        for c in self.cells:
            c.insert(text)

@dataclass
class Text:
    x: Float
    y: Float
    text: str

    def __lt__(self, other) -> bool:
        return other.y < self.y or (self.y == other.y and self.x < other.x)

class Page:
    def __init__(self, n: int, page):
        self.rects = []
        self.hlines = []
        self.vlines = []
        self.text = []
        self.rows = []

        def visitor_operand_before(op, args, cm, tm):
            if op == b"re":
                (x, y, w, h) = (args[i].as_numeric() for i in range(4))
                self.rects.append(Rect(Float(x), Float(y), Float(w), Float(h)))

        def visitor_text(text, cm, tm, fontDict, fontSize):
            (x, y) = (tm[4], tm[5])
            self.text.append(Text(Float(x), Float(y), text))

        page.extract_text(
            visitor_operand_before=visitor_operand_before, visitor_text=visitor_text)

        self.vlines = []
        self.hlines = []
        for rect in self.rects:
            # If the ATS documents were correctly formatted, we could use the
            # rects directly. However in some cases the table does not include
            # a rect for a cell, but instead draws the cell with flat rects
            # for all edges...
            self.vlines.append(VLine(rect.x, rect.y, rect.h))
            self.vlines.append(VLine(rect.x + rect.w, rect.y, rect.h))
            self.hlines.append(HLine(rect.x, rect.y, rect.w))
            self.hlines.append(HLine(rect.x, rect.y + rect.h, rect.w))

        # Discard duplicate segments, and segments of small length
        # (these are generated by the flat rects that draw cell edges).
        unique_vlines = []
        unique_hlines = []
        for l in self.vlines:
            if not l in unique_vlines and not l.l == Float(0.0):
                unique_vlines.append(l)
        for l in self.hlines:
            if not l in unique_hlines and not l.l == Float(0.0):
                unique_hlines.append(l)
        self.vlines = unique_vlines
        self.hlines = unique_hlines
        self.hlines.sort()

        # Try to identify rows of cells in the page.
        hlines = self.hlines.copy()
        while hlines:
            x = hlines[0].x
            y = hlines[0].y
            hl = []
            # 1. match first a sequence of adjoined horizontal segments
            #    with the same y coordinate
            while hlines and hlines[0].y == y:
                assert hlines[0].x == x
                hl.append(hlines.pop(0))
                x = hl[-1].x + hl[-1].l
            # 2. match vertical segments that have the same y coordinate
            #    as origin point
            vl = [l for l in self.vlines if l.y == y]
            if not vl:
                continue
            h = vl[0].l
            # 3. validate that all matched vertical segments have the same
            #    length and match the number of horizontal segments + 1
            assert len(vl) == (len(hl) + 1)
            assert all(l.l == h for l in vl)
            # 4. and voilà
            cells = [Rect(l.x, l.y, l.l, h) for l in hl]
            self.rows.append(Row(cells))

        # Stuff the text in the page in the matched rows of cells.
        for text in self.text:
            for row in self.rows:
                row.insert(text)

        # Reorder the rows so that they match the page order
        # (top to bottom).
        self.rows.reverse()


class Document:
    def __init__(self, pages: List[Page]):
        self.mmis = []
        self.rows = [row for page in pages for row in page.rows]
        previous_matched = False

        for row in self.rows:
            if len(row.cells) != 2:
                continue
            # The text for an MMI name is actually split into multiple bits
            # sometimes including spaces. Just remove them since we don't
            # care about the description.
            left = row.cells[0].contents().replace(' ', '')
            right = row.cells[1].contents().replace(' ', '')
            matched = re.match(r"{(\d+),%s", right)

            if matched:
                mmi_id = int(matched.group(1))
                self.mmis.append((mmi_id, left))
            elif previous_matched and not (right == 'Message' or right.startswith(r'{%d,%s,') or left.endswith('MMIID')):
                # Sometimes the cell describing an MMI is split between
                # two pages of the ATS document, and sometimes again
                # the MMI name it self is split; attempt to recover the
                # full ID by merging cells that do not match the MMI ID pattern
                # with the previous cells.
                mmi_name = self.mmis[-1][1] + left
                self.mmis[-1] = (self.mmis[-1][0], mmi_name.replace(' ', ''))

            previous_matched = matched is not None


def extract_mmis(input: argparse.FileType, page_num: Optional[int] = None) -> list:
    reader = pypdf.PdfReader(input)
    profile = input.name.split('_')[0]
    pages = []
    if page_num is not None:
        pages = [Page(page_num, reader.pages[page_num])]
    elif profile == 'HCRP':
        # The HCRP has a table that does not match the formatting on its last
        # page, but it does not contain any MMI, just skip it.
        pages = [Page(n, page) for (n, page) in enumerate(reader.pages[:-1])]
    else:
        pages = [Page(n, page) for (n, page) in enumerate(reader.pages)]

    document = Document(pages)
    return [(profile, mmi_id, mmi_name) for (mmi_id, mmi_name) in document.mmis]


def run(extract_all: bool, input: Optional[argparse.FileType], page: Optional[int]):

    if not extract_all:
        mmis = extract_mmis(input, page)
    else:
        mmis = []
        pdfs = []
        for name in os.listdir():
            if name.endswith(".pdf"):
                pdfs.append(name)
        pdfs.sort()
        failed_extraction = []
        for pdf in pdfs:
            try:
                print(f"{pdf}", file=sys.stderr)
                mmis.extend(extract_mmis(open(pdf, 'rb')))
            except:
                failed_extraction.append(pdf)
        if failed_extraction:
            print("Failed to extract from:")
            for pdf in failed_extraction:
                print(f" - {pdf}")

    # Patch MMIs
    final_mmis = []
    for mmi in mmis:
        (profile, id, name) = mmi
        if mmi in filtered_mmis:
            continue
        id = patched_mmis.get(mmi, id)
        final_mmis.append((profile, id, name))

    print("// File generated by script/generate_mmi_ids.py. DO NOT EDIT")
    print("match (profile, id) {")
    final_mmis.sort()
    for (profile, mmi_id, mmi_name) in final_mmis:
        print(f"  (\"{profile}\", {mmi_id}) => Some(\"{mmi_name}\"),")
    print("  _ => None")
    print("}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--extract-all', action='store_true', help='Process all pdfs in current dir')
    parser.add_argument('--input', type=argparse.FileType('rb'), help='Input pdf source')
    parser.add_argument('--page', type=int, default=None, help='Page numer')
    return run(**vars(parser.parse_args()))


if __name__ == '__main__':
    sys.exit(main())
