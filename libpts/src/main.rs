use std::cell::RefCell;
use std::env;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::thread;

use dirs;
use libpts::hci::HCIPort;
use libpts::installer;
use libpts::log;
use libpts::mmi;
use libpts::pts;
use libpts::pts::MMIStyle;
use libpts::wine::{Wine, WineArch};
use libpts::xml_model::ets::ETS;
use libpts::xml_model::XMLModel;

const ROOTCANAL_PORT: u16 = 6402;

fn main() {
    let mut cache = dirs::cache_dir().expect("No cache dir");
    cache.push("pts");

    let wine = match Wine::new(cache, WineArch::Win32) {
        Ok(wine) => wine,
        Err(err) => {
            println!("Wine error: {}", err);
            return;
        }
    };

    if installer::is_pts_installation_needed(&wine) {
        println!("Installing PTS");
        installer::install_pts(&wine);
    }

    installer::install_server(&wine).expect("Install server failed");

    println!("Devices {:?}", wine.devices());

    println!("Port {:?}", wine.first_available_com_port());

    let (port, wineport) = HCIPort::bind(&wine).expect("HCI port");

    let mut hcitx = port.clone();
    let mut hcirx = port;

    let tcp = TcpStream::connect((Ipv4Addr::LOCALHOST, ROOTCANAL_PORT)).expect("Connect");

    let mut tcptx = tcp.try_clone().expect("Clone");
    let mut tcprx = tcp;

    thread::spawn(move || io::copy(&mut hcitx, &mut tcprx).expect("HCI TX"));
    thread::spawn(move || io::copy(&mut tcptx, &mut hcirx).expect("HCI RX"));

    let mut dut = Command::new(env::args().nth(1).unwrap())
        .arg("any")
        .env("ROOTCANAL_PORT", ROOTCANAL_PORT.to_string())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("DUT Spawn failed");

    let mut lines = io::BufReader::new(dut.stderr.take().unwrap()).lines();

    let dut_addr = lines
        .next()
        .unwrap()
        .unwrap()
        .replace(":", "")
        .to_uppercase();

    println!("DUT Addr: {}", dut_addr);

    println!("Devices {:?}", wine.devices());

    let parameters = [
        ("TSPC_A2DP_1_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_1_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_5", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_2_6", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_7", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_8", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_9", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_10", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_10a", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_11", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_12", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_13", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_14", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_15", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_16", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2_17", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2a_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2a_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2a_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2b_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_2b_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3_1", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_3_1a", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3_5", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3_6", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3_7", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3_8", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_1", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_3a_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_5", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_6", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_7", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_8", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_9", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_10", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_11", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_3a_12", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_4_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_2", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_4_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_4", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_4_5", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_6", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_7", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_4_8", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_9", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_10", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_4_10a", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_11", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_12", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_4_13", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_4_14", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_4_15", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_5_1", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5_1a", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_5_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_5_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_5_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_5_5", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_5a_1", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_2", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_3", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_4", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_5", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_6", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_7", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_8", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_9", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_10", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_11", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_5a_12", "BOOLEAN", "TRUE"),
        ("TSPC_A2DP_7a_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_7a_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_7a_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_7b_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_7b_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_8_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_8_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_8_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_9_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_9_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_9_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_9_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_10_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_10_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_10_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_10_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_10_5", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_10_6", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_11_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_11_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_11_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_11_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_11_5", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_11_6", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_12_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_12_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_12_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_13_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_13_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_13_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_13_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_14_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_14_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_14_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_14_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_14_5", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_15_1", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_15_2", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_15_3", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_15_4", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_15_5", "BOOLEAN", "FALSE"),
        ("TSPC_A2DP_15_6", "BOOLEAN", "FALSE"),
        ("TSPC_ALL", "BOOLEAN", "FALSE"),
        ("TSPX_security_enabled", "BOOLEAN", "FALSE"),
        ("TSPX_bd_addr_iut", "OCTETSTRING", &dut_addr),
        ("TSPX_SRC_class_of_device", "OCTETSTRING", "080418"),
        ("TSPX_SNK_class_of_device", "OCTETSTRING", "04041C"),
        ("TSPX_pin_code", "IA5STRING", "0000"),
        ("TSPX_delete_link_key", "BOOLEAN", "TRUE"),
        ("TSPX_time_guard", "INTEGER", "300000"),
        ("TSPX_use_implicit_send", "BOOLEAN", "TRUE"),
        (
            "TSPX_media_directory",
            "IA5STRING",
            r"C:\Program Files (x86)\Bluetooth SIG\Bluetooth PTS\bin\audio",
        ),
        ("TSPX_auth_password", "IA5STRING", "0000"),
        ("TSPX_auth_user_id", "IA5STRING", "PTS"),
        ("TSPX_rfcomm_channel", "INTEGER", "8"),
        ("TSPX_l2cap_psm", "OCTETSTRING", "1011"),
        ("TSPX_no_confirmations", "BOOLEAN", "FALSE"),
        ("TSPX_cover_art_uuid", "OCTETSTRING", "3E"),
    ];

    let parameters2 = [
        ("TSPC_A2DP_1_1", false),
        ("TSPC_A2DP_1_2", false),
        ("TSPC_A2DP_2_1", false),
        ("TSPC_A2DP_2_2", false),
        ("TSPC_A2DP_2_3", false),
        ("TSPC_A2DP_2_4", false),
        ("TSPC_A2DP_2_5", true),
        ("TSPC_A2DP_2_6", false),
        ("TSPC_A2DP_2_7", false),
        ("TSPC_A2DP_2_8", false),
        ("TSPC_A2DP_2_9", false),
        ("TSPC_A2DP_2_10", false),
        ("TSPC_A2DP_2_10a", false),
        ("TSPC_A2DP_2_11", false),
        ("TSPC_A2DP_2_12", false),
        ("TSPC_A2DP_2_13", false),
        ("TSPC_A2DP_2_14", false),
        ("TSPC_A2DP_2_15", false),
        ("TSPC_A2DP_2_16", false),
        ("TSPC_A2DP_2_17", false),
        ("TSPC_A2DP_2a_1", false),
        ("TSPC_A2DP_2a_2", false),
        ("TSPC_A2DP_2a_3", false),
        ("TSPC_A2DP_2b_1", false),
        ("TSPC_A2DP_2b_2", false),
        ("TSPC_A2DP_3_1", true),
        ("TSPC_A2DP_3_1a", false),
        ("TSPC_A2DP_3_2", false),
        ("TSPC_A2DP_3_3", false),
        ("TSPC_A2DP_3_4", false),
        ("TSPC_A2DP_3_5", false),
        ("TSPC_A2DP_3_6", false),
        ("TSPC_A2DP_3_7", false),
        ("TSPC_A2DP_3_8", false),
        ("TSPC_A2DP_3a_1", true),
        ("TSPC_A2DP_3a_2", false),
        ("TSPC_A2DP_3a_3", false),
        ("TSPC_A2DP_3a_4", false),
        ("TSPC_A2DP_3a_5", false),
        ("TSPC_A2DP_3a_6", false),
        ("TSPC_A2DP_3a_7", false),
        ("TSPC_A2DP_3a_8", false),
        ("TSPC_A2DP_3a_9", false),
        ("TSPC_A2DP_3a_10", false),
        ("TSPC_A2DP_3a_11", false),
        ("TSPC_A2DP_3a_12", true),
        ("TSPC_A2DP_4_1", false),
        ("TSPC_A2DP_4_2", true),
        ("TSPC_A2DP_4_3", false),
        ("TSPC_A2DP_4_4", true),
        ("TSPC_A2DP_4_5", false),
        ("TSPC_A2DP_4_6", false),
        ("TSPC_A2DP_4_7", true),
        ("TSPC_A2DP_4_8", false),
        ("TSPC_A2DP_4_9", false),
        ("TSPC_A2DP_4_10", true),
        ("TSPC_A2DP_4_10a", false),
        ("TSPC_A2DP_4_11", false),
        ("TSPC_A2DP_4_12", false),
        ("TSPC_A2DP_4_13", true),
        ("TSPC_A2DP_4_14", true),
        ("TSPC_A2DP_4_15", false),
        ("TSPC_A2DP_5_1", true),
        ("TSPC_A2DP_5_1a", false),
        ("TSPC_A2DP_5_2", false),
        ("TSPC_A2DP_5_3", false),
        ("TSPC_A2DP_5_4", false),
        ("TSPC_A2DP_5_5", false),
        ("TSPC_A2DP_5a_1", true),
        ("TSPC_A2DP_5a_2", true),
        ("TSPC_A2DP_5a_3", true),
        ("TSPC_A2DP_5a_4", true),
        ("TSPC_A2DP_5a_5", true),
        ("TSPC_A2DP_5a_6", true),
        ("TSPC_A2DP_5a_7", true),
        ("TSPC_A2DP_5a_8", true),
        ("TSPC_A2DP_5a_9", true),
        ("TSPC_A2DP_5a_10", true),
        ("TSPC_A2DP_5a_11", true),
        ("TSPC_A2DP_5a_12", true),
        ("TSPC_A2DP_7a_1", false),
        ("TSPC_A2DP_7a_2", false),
        ("TSPC_A2DP_7a_3", false),
        ("TSPC_A2DP_7b_1", false),
        ("TSPC_A2DP_7b_2", false),
        ("TSPC_A2DP_8_2", false),
        ("TSPC_A2DP_8_3", false),
        ("TSPC_A2DP_8_4", false),
        ("TSPC_A2DP_9_1", false),
        ("TSPC_A2DP_9_2", false),
        ("TSPC_A2DP_9_3", false),
        ("TSPC_A2DP_9_4", false),
        ("TSPC_A2DP_10_1", false),
        ("TSPC_A2DP_10_2", false),
        ("TSPC_A2DP_10_3", false),
        ("TSPC_A2DP_10_4", false),
        ("TSPC_A2DP_10_5", false),
        ("TSPC_A2DP_10_6", false),
        ("TSPC_A2DP_11_1", false),
        ("TSPC_A2DP_11_2", false),
        ("TSPC_A2DP_11_3", false),
        ("TSPC_A2DP_11_4", false),
        ("TSPC_A2DP_11_5", false),
        ("TSPC_A2DP_11_6", false),
        ("TSPC_A2DP_12_2", false),
        ("TSPC_A2DP_12_3", false),
        ("TSPC_A2DP_12_4", false),
        ("TSPC_A2DP_13_1", false),
        ("TSPC_A2DP_13_2", false),
        ("TSPC_A2DP_13_3", false),
        ("TSPC_A2DP_13_4", false),
        ("TSPC_A2DP_14_1", false),
        ("TSPC_A2DP_14_2", false),
        ("TSPC_A2DP_14_3", false),
        ("TSPC_A2DP_14_4", false),
        ("TSPC_A2DP_14_5", false),
        ("TSPC_A2DP_15_1", false),
        ("TSPC_A2DP_15_2", false),
        ("TSPC_A2DP_15_3", false),
        ("TSPC_A2DP_15_4", false),
        ("TSPC_A2DP_15_5", false),
        ("TSPC_A2DP_15_6", false),
        ("TSPC_ALL", false),
    ];

    // WIP
    let mut ets: ETS = ETS::parse(String::from("A2DP"), &wine).unwrap_or_else(|err| {
        println!("{}", err);
        std::process::exit(2);
    });
    println!(
        "{:?}",
        ets.get_valid_testcases(&parameters2).collect::<Vec<_>>()
    );
    // END WIP

    let faddr: Rc<RefCell<String>> = Rc::new(RefCell::new("".to_string()));
    let faddr_clone = Rc::clone(&faddr);

    let mut stdin = dut.stdin.take().unwrap();

    let messages = pts::run(
        wineport,
        "A2DP",
        "A2DP/SNK/AS/BV-01-I",
        parameters.iter(),
        move |mmi, style| {
            let (id, test, profile, description) = mmi::parse(mmi).unwrap();

            let values = match style {
                MMIStyle::OkCancel1 | MMIStyle::OkCancel2 => "2|OK|Cancel",
                MMIStyle::Ok => "1|OK",
                MMIStyle::YesNo1 => "2|Yes|No",
                MMIStyle::YesNoCancel1 => "3|Yes|No|Cancel",
                MMIStyle::AbortRetry1 => "3|Abort|Retry|Ignore",
                MMIStyle::Edit1 => "0",
                MMIStyle::Edit2 => unreachable!(),
            };

            write!(
                &mut stdin,
                "any|{addr}|{id}|{test}|{values}|{description}\0",
                addr = faddr_clone.borrow(),
                id = mmi::id_to_mmi(&profile, id).unwrap_or(&id.to_string()),
                test = test,
                values = values,
                description = description
            )
            .unwrap();

            stdin.flush().unwrap();

            let answer = lines.next().unwrap().unwrap();

            answer
        },
    );

    let messages = messages.map(|r| r.unwrap());

    let (addr, events) = log::parse(messages);

    faddr.replace(format!(
        "{}:{}:{}:{}:{}:{}",
        &addr[0..2],
        &addr[2..4],
        &addr[4..6],
        &addr[6..8],
        &addr[8..10],
        &addr[10..12]
    ));

    for event in events {
        match event {
            log::Event::EnterTestStep(test_step, num) => {
                println!("{:<1$}{test_step}", "", num * 2, test_step = test_step)
            }
            log::Event::Message(
                pts::Message::Log {
                    ref message,
                    ref description,
                    ..
                },
                num,
            ) => {
                println!(
                    "{:<1$}- {description}{message}",
                    "",
                    num * 2,
                    description = description.trim(),
                    message = message.trim()
                );
            }
            _ => {}
        }
    }
}
