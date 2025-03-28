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

from setuptools import setup, Command
from setuptools.command.build_py import build_py
import pkg_resources
import os

package_directory = os.path.dirname(os.path.realpath(__file__))

os.environ["PATH"] = package_directory + ':' + os.environ["PATH"]


class BuildGrpc(Command):
    """GRPC build command."""
    description = 'build grpc files'
    user_options = []

    def initialize_options(self):
        pass

    def finalize_options(self):
        pass

    def run(self):
        from grpc_tools import protoc

        proto_include = pkg_resources.resource_filename('grpc_tools', '_proto')

        files = [f'blueberry/{f}' for f in os.listdir('proto/blueberry') if f.endswith('.proto')]
        protoc.main([
            'grpc_tools.protoc',
            '-Iproto',
            f'-I{proto_include}',
            '--python_out=.',
            '--custom_grpc_out=.',
        ] + files)


class BuildPyCommand(build_py):
    """Custom build command."""

    def run(self):
        self.run_command('build_grpc')
        build_py.run(self)


setup(
    name='mmi2grpc',
    version='0.0.1',
    packages=['mmi2grpc', 'blueberry'],
    install_requires=[
        'grpcio',
    ],
    setup_requires=[
        'grpcio-tools'
    ],
    cmdclass={
        'build_grpc': BuildGrpc,
        'build_py': BuildPyCommand,
    }
)
