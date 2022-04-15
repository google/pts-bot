from typing import List
import grpc
import time
import sys

from blueberry.host_grpc import Host

from .a2dp import A2DPProxy
from ._description import format_proxy

GRPC_PORT = 8999


class IUT:
    def __init__(
            self, test: str, args: List[str], port: int = GRPC_PORT, **kwargs):
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
                return Host(channel).ReadLocalAddress(
                    wait_for_ready=True).address
            except grpc.RpcError:
                print('Retry')
                time.sleep(5)
                return Host(channel).ReadLocalAddress(
                    wait_for_ready=True).address

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
                self.a2dp_ = A2DPProxy(
                    grpc.insecure_channel(f'localhost:{self.port}'))
            return self.a2dp_.interact(
                interaction, test, description, pts_address)

        code = format_proxy(profile, interaction, description)
        error_msg = (
            f'Missing {profile} proxy and mmi: {interaction}\n'
            f'Create a {profile.lower()}.py in mmi2grpc/:\n\n{code}\n'
            f'Then, instantiate the corresponding proxy in __init__.py\n'
            f'Finally, create a {profile.lower()}.proto in proto/blueberry/'
            f'and generate the corresponding interface.'
        )
        assert False, error_msg
