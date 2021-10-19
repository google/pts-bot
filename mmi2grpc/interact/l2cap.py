import os

from grpc import Channel

from facade import l2cap_grpc, neighbor_grpc
from facade.common_pb2 import BluetoothAddress
from facade.l2cap_pb2 import RetransmissionFlowControlMode

PSM = 1 # TODO: Add it to either utils.py or config file

def interact(channel: Channel, interaction_id: str, pts_addr: bytes):
    print(f'mmi_id: {interaction_id}')
    addr = BluetoothAddress(address=pts_addr)
    l2cap = l2cap_grpc.L2capClassicModuleFacade(channel)
    neighbor = neighbor_grpc.NeighborFacade(channel)
    if interaction_id == "MMI_TESTER_ENABLE_CONNECTION":
        neighbor.EnablePageScan(enabled=True)
        l2cap.SetDynamicChannel(
            psm=PSM,
            enable=True,
            retransmission_mode=RetransmissionFlowControlMode.BASIC
        )
    if interaction_id == "MMI_IUT_SEND_CONFIG_REQ":
        pass
    if interaction_id == "MMI_IUT_SEND_L2CAP_DATA":
        payload = b'\x00' + os.urandom(40) + b'\x00'
        l2cap.SendDynamicChannelPacket(
            remote=addr,
            psm=PSM,
            payload=payload
        )
    if interaction_id == "MMI_IUT_INITIATE_ACL_CONNECTION":
        l2cap.SetDynamicChannel(
            psm=PSM,
            enable=True,
            retransmission_mode=RetransmissionFlowControlMode.BASIC
        )
        l2cap.OpenChannel(
            remote=addr,
            psm=PSM,
            mode=RetransmissionFlowControlMode.BASIC
        )
    if interaction_id == ("MMI_IUT_DISABLE_CONNECTION" or "MMI_IUT_SEND_DISCONNECT_RSP"):
        l2cap.CloseChannel(psm=PSM)
    if interaction_id == "MMI_IUT_SEND_ACL_DISCONNECTON":
        pass
    if interaction_id == "MMI_IUT_SEND_CONFIG_RSP":
        pass
