pub mod facade {
    tonic::include_proto!("bluetooth.facade");
}

pub mod l2cap {
    pub mod classic {
        tonic::include_proto!("bluetooth.l2cap.classic");
    }
}

pub mod neighbor {
    tonic::include_proto!("bluetooth.neighbor");
}
