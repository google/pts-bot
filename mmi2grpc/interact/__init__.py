import grpc
from . import l2cap

GRPC_PORT = 8999

def run(profile: str, interaction_id: str, pts_addr: bytes):
    channel = grpc.insecure_channel(f'localhost:{GRPC_PORT}')
    if profile == "L2CAP":
        l2cap.interact(channel, interaction_id, pts_addr)
