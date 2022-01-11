from typing import Optional
import grpc
import time
import sys
import textwrap

from .a2dp import A2DPProxy

from blueberry.host_grpc import Host

GRPC_PORT = 8999

_a2dp: Optional[A2DPProxy] = None


def run(profile: str, interaction_id: str, test: str, description: str, pts_addr: bytes):
    global _a2dp
    print(f'{profile} mmi: {interaction_id}', file=sys.stderr)
    if profile in ('A2DP', 'AVDTP'):
        if not _a2dp:
            _a2dp = A2DPProxy(grpc.insecure_channel(f'localhost:{GRPC_PORT}'))
        return _a2dp.interact(interaction_id, test, description, pts_addr)


def reset():
    global _a2dp
    _a2dp = None
    with grpc.insecure_channel(f'localhost:{GRPC_PORT}') as channel:
        Host(channel).Reset(wait_for_ready=True)


def read_local_address() -> bytes:
    with grpc.insecure_channel(f'localhost:{GRPC_PORT}') as channel:
        try:
            return Host(channel).ReadLocalAddress(wait_for_ready=True).address
        except grpc.RpcError:
            print('Retry')
            time.sleep(5)
            return Host(channel).ReadLocalAddress(wait_for_ready=True).address
