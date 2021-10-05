import os

from grpc import Channel
from lib.proto import l2cap_pb2_grpc
from lib.proto import neighbor_pb2_grpc

from lib.proto.common_pb2 import BluetoothAddress
from lib.proto.l2cap_pb2 import CloseChannelRequest, OpenChannelRequest, SetEnableDynamicChannelRequest, RetransmissionFlowControlMode, DynamicChannelPacket, OpenChannelRequest
from lib.proto.neighbor_pb2 import EnableMsg
PSM = 1 # TODO: Add it to either utils.py or config file


def interact(channel: Channel, interaction_id: str, pts_addr: bytes):
    print(f'mmi_id: {interaction_id}')
    addr = BluetoothAddress(address=pts_addr)
    l2cap = l2cap_pb2_grpc.L2capClassicModuleFacadeStub(channel)
    neighbor = neighbor_pb2_grpc.NeighborFacadeStub(channel)
    if interaction_id == "MMI_TESTER_ENABLE_CONNECTION":
        neighbor.EnablePageScan(EnableMsg(enabled=True))
        l2cap.SetDynamicChannel(
            SetEnableDynamicChannelRequest(
                psm=PSM,
                enable=True,
                retransmission_mode=RetransmissionFlowControlMode.BASIC
            )
        )
    if interaction_id == "MMI_IUT_SEND_CONFIG_REQ":
        pass
    if interaction_id == "MMI_IUT_SEND_L2CAP_DATA":
        payload = b'\x00' + os.urandom(40) + b'\x00'
        l2cap.SendDynamicChannelPacket(
            DynamicChannelPacket(
                remote=addr,
                psm=PSM,
                payload=payload
            )
        )
    if interaction_id == "MMI_IUT_INITIATE_ACL_CONNECTION":
        l2cap.SetDynamicChannel(
            SetEnableDynamicChannelRequest(
                psm=PSM,
                enable=True,
                retransmission_mode=RetransmissionFlowControlMode.BASIC
            )
        )
        l2cap.OpenChannel(
            OpenChannelRequest(
                remote=addr,
                psm=PSM,
                mode=RetransmissionFlowControlMode.BASIC
            )
        )
    if interaction_id == ("MMI_IUT_DISABLE_CONNECTION" or "MMI_IUT_SEND_DISCONNECT_RSP"):
        print(f'Sending CLOSE CHANNEL')
        l2cap.CloseChannel(CloseChannelRequest(psm=PSM))
    if interaction_id == "MMI_IUT_SEND_ACL_DISCONNECTON":
        pass
    if interaction_id == "MMI_IUT_SEND_CONFIG_RSP":
        pass
