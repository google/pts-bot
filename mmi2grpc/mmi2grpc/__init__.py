# Copyright 2022 Google LLC

from typing import List
import grpc
import time
import sys
import textwrap

from .a2dp import A2DPProxy

from blueberry.host_grpc import Host

GRPC_PORT = 8999


class IUT:
    def __init__(self, test: str, args: List[str],
                 port: int = GRPC_PORT, **kwargs):
        self.a2dp_ = None
        self.address_ = None
        self.port = port
        self.test = test

    def __enter__(self):
        with grpc.insecure_channel(f'localhost:{self.port}') as channel:
            Host(channel).Reset(wait_for_ready=True)

    def __exit__(self):
        self.a2dp_ = None

    @property
    def address(self) -> bytes:
        with grpc.insecure_channel(f'localhost:{self.port}') as channel:
            try:
                return Host(channel).ReadLocalAddress(wait_for_ready=True).address
            except grpc.RpcError:
                print('Retry')
                time.sleep(5)
                return Host(channel).ReadLocalAddress(wait_for_ready=True).address

    def interact(self,
                 pts_address: bytes,
                 profile: str,
                 test: str,
                 interaction: str,
                 description: str,
                 style: str,
                 **kwargs) -> str:
        print(f'{profile} mmi: {interaction}', file=sys.stderr)
        if profile in ('A2DP', 'AVDTP'):
            if not self.a2dp_:
                self.a2dp_ = A2DPProxy(grpc.insecure_channel(f'localhost:{self.port}'))
            return self.a2dp_.interact(interaction, test, description, pts_address)
