#!/usr/bin/env python3
from setuptools import setup, Command
from setuptools.command.build_py import build_py
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

        protoc.main([
            'grpc_tools.protoc',
            '-I=proto',
            '--python_out=.',
            '--custom_grpc_out=.',
            'facade/l2cap.proto',
            'facade/neighbor.proto',
            'facade/common.proto',
            'facade/rootservice.proto',
        ])


class BuildPyCommand(build_py):
    """Custom build command."""

    def run(self):
        self.run_command('build_grpc')
        build_py.run(self)


setup(
    name='mmi2grpc',
    version='0.0.1',
    packages=['interact', 'facade'],
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
