from grpc import Channel

from blueberry.a2dp_grpc import A2DP
from blueberry.host_grpc import Host

from blueberry.a2dp_pb2 import Sink, Source
from blueberry.host_pb2 import Connection

_connection: Connection = None;
_sink: Sink = None;
_source: Source = None;

def interact(channel: Channel, interaction_id: str, test: str, pts_addr: bytes):
    global _connection, _sink, _source;
    a2dp = A2DP(channel)
    host = Host(channel)
    if interaction_id == "TSC_AVDTP_mmi_iut_accept_connect":
        host.SetConnectable(connectable=True)
    elif interaction_id == "TSC_AVDTP_mmi_iut_initiate_start":
        _connection = host.Connect(address=pts_addr).connection
        if "SNK" in test:
            _sink = a2dp.OpenSink(connection=_connection).sink
    elif interaction_id == "TSC_AVDTP_mmi_iut_initiate_out_of_range":
        if not _connection:
            _connection = Connection(cookie=pts_addr)
        host.Disconnect(connection=_connection)
        _connection = None
        _sink = None
        _source = None
    elif interaction_id == "TSC_AVDTP_mmi_iut_accept_discover":
        pass
    elif interaction_id == "TSC_AVDTP_mmi_iut_initiate_set_configuration":
        _connection = host.Connect(address=pts_addr).connection
        if "SRC" in test:
            _source = a2dp.OpenSource(connection=_connection).source
    elif interaction_id == "TSC_AVDTP_mmi_iut_accept_close_stream":
        pass
    elif interaction_id == "TSC_AVDTP_mmi_iut_accept_get_capabilities":
        pass
    elif interaction_id == "TSC_AVDTP_mmi_iut_accept_set_configuration":
        pass
    elif interaction_id == "TSC_AVDTP_mmi_iut_accept_open_stream":
        pass
    elif interaction_id == "TSC_AVDTP_mmi_iut_accept_start":
        pass
    elif interaction_id == "TSC_AVDTP_mmi_iut_confirm_streaming":
        pass
    elif interaction_id == "TSC_AVDTP_mmi_iut_accept_reconnect":
        pass
    else:
        print(f'MMI NOT IMPLEMENTED: {interaction_id}')