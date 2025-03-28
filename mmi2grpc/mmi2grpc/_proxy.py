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

from mmi2grpc._description import format_function


class ProfileProxy:

    def interact(self, id: str, test: str, description: str, pts_addr: bytes):
        try:
            return getattr(self, id)(
                test=test, description=description, pts_addr=pts_addr)
        except AttributeError:
            code = format_function(id, description)
            assert False, f'Unhandled mmi {id}\n{code}'
