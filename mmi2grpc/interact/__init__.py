import grpc
from . import a2dp

from blueberry.host_grpc import Host

GRPC_PORT = 8999

def run(profile: str, interaction_id: str, test: str, pts_addr: bytes):
    channel = grpc.insecure_channel(f'localhost:{GRPC_PORT}')
    print(f'{profile} mmi: {interaction_id}')
    if profile == "A2DP":
        a2dp.interact(channel, interaction_id, test, pts_addr)
    channel.close()

def read_local_address() -> bytes:
    channel = grpc.insecure_channel(f'localhost:{GRPC_PORT}')
    bluetooth_address = Host(channel).ReadLocalAddress(wait_for_ready=True)
    channel.close()
    return bluetooth_address.address
