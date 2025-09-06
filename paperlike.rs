use std::env;
use std::time::Duration;
use std::io::{Read, Write};
use serialport::SerialPort;

const COM_STR_SIZE: usize = 24;

/* commands */
const COM_PREFIX: u16 = 0x5FF5;
const COM_TAIL: u16 = 0xA0FA;
const COM_CMD_CONTRAST: u8 = 0x01;
const COM_CMD_MODE: u8 = 0x02;
const COM_CMD_REFRESH: u8 = 0x03;

/* this command stores value but doesn't change anything noticible */
/* probably an artifact which worked for the legacy monitors */
/* const COM_CMD_SPEED: u8 = 0x04; */

/* this is supposed to set/sync monitor's internal clock but is not needed for paperlike 13K monitor */
/* const COM_CMD_RTC: u8 = 0x05; */

/* this sets fronlight mode like 0 (no light) 1 (white) 2 (yellow) 3(white again) */
/* but is not needed as below TEMPERATURE command handles that */
/* const COM_CMD_FRONTLIGHT: u8 = 0x07; */

/* this sets frontlight gamma/temperature using 0-100 (yellow to white) */
/* but is weird a little as results are not smooth e.g. has discreet ranges of the same effect: */
/* 0-1, 2-21, 22-24, 25-41, 42-49, 50-60, 61-73, 74-80, 81-98, 99-100 */
/* hence there are 10 discreet levels of temperature [0,2,22,25,42,50,61,74,81,99]*/
const COM_CMD_FL_TEMPERATURE: u8 = 0x08;

// brightness 0-100 (dim-bright)
const COM_CMD_FL_LEVEL: u8 = 0x09;

const COM_CMD_GET_PARAMETERS: u8 = 0x0A;
const COM_GET_VERSION: u8 = 0x10;

// this seems like the version of Paperlike13K protocol, for 253 monitor this can be 0x10, etc.
// this is used to identify correct usb port
const PROTOCOL_VERSION: u8 = 0x31;

// temperature discrete levels mapping (1-10 maps to these actual hardware values)
const TEMPERATURE_LEVELS: [u8; 10] = [0, 2, 22, 25, 42, 50, 61, 74, 81, 99];

// convert raw temperature value back to user-friendly level (1-10)
fn raw_temp_to_level(raw_value: u32) -> u32 {
    for (index, &threshold) in TEMPERATURE_LEVELS.iter().enumerate() {
        if raw_value <= threshold as u32 {
            return (index + 1) as u32;
        }
    }
    // if value is higher than the highest threshold, return level 10
    10
}

enum Action {
    Temperature(i32),
    Frontlight(i32),
    Mode(i32),
    Contrast(i32),
    Refresh,
    Info,
    Help,
}

fn open_serial_port(port: &str) -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
    let port = serialport::new(port, 115200)
        .timeout(Duration::from_secs(10))
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .flow_control(serialport::FlowControl::None)
        .open()?;
    
    Ok(port)
}

fn set_paperlike_parameters(port_name: &str, cmd_type: u8, value: i32) {
    match open_serial_port(port_name) {
        Ok(mut port) => {
            let buf = format!("{:04X}{:02X}{:02X}{:012X}{:04X}", 
                COM_PREFIX, cmd_type, value & 0xFF, 0, COM_TAIL);
            
            match port.write_all(buf.as_bytes()) {
                Ok(_) => {},
                Err(_) => println!("set parameter error"),
            }
        }
        Err(e) => {
            println!("uart_open : can't open serial port");
            println!("open {} error: {}", port_name, e);
        }
    }
}

fn get_paperlike_parameters(port_name: &str, cmd: u8) -> i32 {
    match open_serial_port(port_name) {
        Ok(mut port) => {
            let buf = format!("{:04X}{:02X}{:02X}{:012X}{:04X}", 
                COM_PREFIX, COM_CMD_GET_PARAMETERS, cmd & 0xFF, 0, COM_TAIL);
            
            if let Err(_) = port.write_all(buf.as_bytes()) {
                println!("cannot send command");
                return 0;
            }
            let mut rcv_buf = [0u8; COM_STR_SIZE];
            match port.read(&mut rcv_buf) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let rcv_str = String::from_utf8_lossy(&rcv_buf[..bytes_read]);
                        // parse the response
                        if rcv_str.len() >= 12 {
                            let cmd_parsed = u32::from_str_radix(&rcv_str[4..8], 16);
                            let data1 = u32::from_str_radix(&rcv_str[8..10], 16);
                            let data2 = u32::from_str_radix(&rcv_str[10..12], 16);
                            // println!("parsed response: cmd={:?}, data1={:?}, data2={:?}", cmd_parsed, data1, data2);
                            if let (Ok(_cmd_parsed), Ok(data1), Ok(data2)) = (cmd_parsed, data1, data2) {
                                match data1 as u8 {
                                    COM_GET_VERSION => println!("protocol version    :  0x{:x}", data2),
                                    COM_CMD_CONTRAST => println!("contrast    :  {} (1-9)", data2),
                                    COM_CMD_MODE => {
                                        // hardware quirk?: mode=1 when set returns value 5 when queried
                                        let display_mode = if data2 == 5 { 1 } else { data2 };
                                        println!("mode        :  {} (1-4)", display_mode);
                                    },
                                    COM_CMD_FL_TEMPERATURE => {
                                        let level = raw_temp_to_level(data2);
                                        println!("temperature :  {} (1-10)", level);
                                    },
                                    COM_CMD_FL_LEVEL => println!("frontlight  :  {} (0-100)", data2),
                                    _ => println!("get info error: {} {} {}", _cmd_parsed, data1, data2),
                                }
                                return data2 as i32;
                            }
                        }
                    }
                    println!("not send ACK, just return");
                    0
                }
                Err(_) => {
                    println!("can not receive");
                    0
                }
            }
        }
        Err(e) => {
            println!("open {} error: {}", port_name, e);
            0
        }
    }
}

fn port_detection() {
    println!("detecting Paperlike devices on USB serial ports(0-10)...");
    let mut found_devices = false;
    for i in 0..10 {
        let port_name = format!("/dev/ttyUSB{}", i);
        if std::path::Path::new(&port_name).exists() {
            println!("checking port: {}", port_name);
            let ret = get_paperlike_parameters(&port_name, COM_GET_VERSION);
            if ret == PROTOCOL_VERSION as i32 {
                println!("compatible paperlike device found at: {}", port_name);
                println!("you can use: 'sudo ./paperlike {} -<command>'", port_name);
                // println!("or use /dev/paperlike if udev rule was created'");
                print_help();
                found_devices = true;
            } else if ret != 0 {
                println!("?unknown device at {} (version: 0x{:x})", port_name, ret);
            }
        }
    }
    if !found_devices {
        println!("no compatible Paperlike devices found");
        println!("make sure your device is connected and you have permission to access serial ports");
    }
}

fn parse_args(args: Vec<String>) -> Result<(String, Action), &'static str> {
    if args.len() < 3 {
        return Err("wrong number of arguments");
    }
    
    let port = args[1].clone();
    
    if !args[2].starts_with('-') {
        return Err("invalid argument format");
    }
    
    let flag = &args[2][1..];
    
    match flag {
        "r" => {
            if args.len() != 3 {
                return Err("too many arguments for refresh flag");
            }
            Ok((port, Action::Refresh))
        }
        "i" => {
            if args.len() != 3 {
                return Err("too many arguments for info flag");
            }
            Ok((port, Action::Info))
        }
        "h" => {
            if args.len() != 3 {
                return Err("too many arguments for help flag");
            }
            Ok((port, Action::Help))
        }
        "t" | "f" | "m" | "c" => {
            if args.len() != 4 {
                return Err("missing value for flag");
            }
            
            let value: i32 = args[3].parse().map_err(|_| "invalid numeric value")?;
            
            match flag {
                "t" => {
                    if !(1..=10).contains(&value) {
                        println!(" temperature: 1-10 ");
                        return Err("invalid temperature range");
                    }
                    let actual_temp_value = TEMPERATURE_LEVELS[(value - 1) as usize] as i32;
                    Ok((port, Action::Temperature(actual_temp_value)))
                }
                "f" => {
                    if !(0..=100).contains(&value) {
                        println!(" frontlight level 0-100 ");
                        return Err("invalid frontlight range");
                    }
                    Ok((port, Action::Frontlight(value)))
                }
                "m" => {
                    if !(1..=4).contains(&value) {
                        println!(" 1-4 ");
                        return Err("invalid mode range");
                    }
                    Ok((port, Action::Mode(value)))
                }
                "c" => {
                    if !(1..=9).contains(&value) {
                        println!(" 1-9 ");
                        return Err("invalid contrast range");
                    }
                    Ok((port, Action::Contrast(value)))
                }
                _ => unreachable!(),
            }
        }
        _ => Err("unknown flag"),
    }
}

fn print_help() {
    println!("usage:");
    println!(" sudo paperlike /dev/ttyUSB* -<command> <opt val>");
    println!("if udev rule has been created and /dev/paperlike linked:");
    println!(" paperlike /dev/paperlike -<command> <opt val>");
    println!("input parameters:");
    println!("   /dev/ttyUSB*: device serial port");
    println!("   -<command> <opt val>:");
    println!("     -i      device information");
    println!("     -r      refresh screen");
    println!("     -c  3   contrast, options 1-9");
    println!("     -m  1   mode, options 1-4");
    println!("     -t  4   temperature (1-10)");
    println!("     -f  3   frontlight level (0-100)");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // If no arguments provided, run device detection only
    if args.len() == 1 {
        port_detection();
        return;
    }
    let (port, action) = match parse_args(args.clone()) {
        Ok((p, a)) => (p, a),
        Err(e) => {
            eprintln!("error: {}", e);
            print_help();
            return;
        }
    };
    match action {
        Action::Help => {
            print_help();
        }
        Action::Refresh => {
            set_paperlike_parameters(&port, COM_CMD_REFRESH, 0);
        }
        Action::Info => {
            get_paperlike_parameters(&port, COM_CMD_CONTRAST);
            get_paperlike_parameters(&port, COM_CMD_MODE);
            get_paperlike_parameters(&port, COM_CMD_FL_TEMPERATURE);
            get_paperlike_parameters(&port, COM_CMD_FL_LEVEL);
        }
        Action::Temperature(value) => {
            set_paperlike_parameters(&port, COM_CMD_FL_TEMPERATURE, value);
        }
        Action::Frontlight(value) => {
            set_paperlike_parameters(&port, COM_CMD_FL_LEVEL, value);
        }
        Action::Mode(value) => {
            set_paperlike_parameters(&port, COM_CMD_MODE, value);
        }
        Action::Contrast(value) => {
            set_paperlike_parameters(&port, COM_CMD_CONTRAST, value);
        }
    }
}
