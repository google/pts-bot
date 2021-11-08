#!/usr/bin/env python3
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

        protoc.main([
            'grpc_tools.protoc',
            '-Iproto',
            f'-I{proto_include}',
            '--python_out=.',
            '--custom_grpc_out=.',
            'blueberry/host.proto',
            'blueberry/a2dp.proto'
        ])


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
