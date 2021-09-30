use super::bluetooth::facade::BluetoothAddress;
use super::bluetooth::l2cap::classic::{
    l2cap_classic_module_facade_client::L2capClassicModuleFacadeClient, CloseChannelRequest,
    DynamicChannelPacket, OpenChannelRequest, RetransmissionFlowControlMode,
    SetEnableDynamicChannelRequest,
};
use super::bluetooth::neighbor::{neighbor_facade_client::NeighborFacadeClient, EnableMsg};

use super::Interaction;
use tonic::Request;

const GRPC_ADDR: &'static str = "http://127.0.0.1:8999";
const PSM: u32 = 1;

macro_rules! grpc_call {
    ($client: ident, $method: ident, $request: expr) => {
        let request = Request::new($request);
        let mut client = $client::connect(GRPC_ADDR.to_string()).await?;
        client.$method(request).await?;
    };
}

pub async fn handle(interaction: Interaction<'_>) -> Result<String, Box<dyn std::error::Error>> {
    match interaction.id {
        "MMI_TESTER_ENABLE_CONNECTION" => {
            grpc_call!(
                NeighborFacadeClient,
                enable_page_scan,
                EnableMsg { enabled: true }
            );
            grpc_call!(
                L2capClassicModuleFacadeClient,
                set_dynamic_channel,
                SetEnableDynamicChannelRequest {
                    psm: PSM,
                    enable: true,
                    retransmission_mode: RetransmissionFlowControlMode::Basic as i32,
                }
            );
        }
        "MMI_IUT_SEND_CONFIG_REQ" => {}
        "MMI_IUT_SEND_L2CAP_DATA" => {
            let pts_addr = BluetoothAddress {
                address: format!("{}", interaction.pts_addr).into_bytes(),
            };
            grpc_call!(
                L2capClassicModuleFacadeClient,
                send_dynamic_channel_packet,
                DynamicChannelPacket {
                    remote: Some(pts_addr),
                    psm: PSM,
                    payload: (0..40).map(|_| rand::random::<u8>()).collect(),
                }
            );
        }
        "MMI_IUT_INITIATE_ACL_CONNECTION" => {
            grpc_call!(
                L2capClassicModuleFacadeClient,
                set_dynamic_channel,
                SetEnableDynamicChannelRequest {
                    psm: PSM,
                    enable: true,
                    retransmission_mode: RetransmissionFlowControlMode::Basic as i32,
                }
            );
            let pts_addr = BluetoothAddress {
                address: format!("{}", interaction.pts_addr).into_bytes(),
            };
            grpc_call!(
                L2capClassicModuleFacadeClient,
                open_channel,
                OpenChannelRequest {
                    remote: Some(pts_addr),
                    psm: PSM,
                    mode: RetransmissionFlowControlMode::Basic as i32,
                }
            );
        }
        "MMI_IUT_DISABLE_CONNECTION" | "MMI_IUT_SEND_DISCONNECT_RSP" => {
            grpc_call!(
                L2capClassicModuleFacadeClient,
                close_channel,
                CloseChannelRequest { psm: PSM }
            );
        }
        "MMI_IUT_SEND_ACL_DISCONNECTION" => {}
        "MMI_IUT_SEND_CONFIG_RSP" => {}
        _ => {
            println!("id: {}", interaction.id);
            todo!();
        }
    }
    Ok(String::from("Ok"))
}
