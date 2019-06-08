extern crate hidapi;

use hidapi::HidApi;
use std::fs;
use std::io;
use std::io::Read;

const VID: u16 = 0x0000; // CHANGE ME TODO
const PID: u16 = 0x0000; // CHANGE ME TODO

fn upload_code(path: &str, api: HidApi) -> io::Result<()> {
    let f = fs::File::open(path)?;
    let mut cmds: Vec<Vec<u8>> = Vec::new();
    let iter = &mut f.bytes().peekable();
    println!("Parsing commands");
    loop {
        if let Some(Ok(0x24u8)) = iter.peek() {
            print!(".");
            let mut arr: Vec<u8> = Vec::new();
            arr.push(iter.next().unwrap().unwrap());
            let num_bytes = iter.next().unwrap().unwrap();
            arr.push(num_bytes);
            arr.extend(iter.by_ref().take(num_bytes as usize).map(|c| c.unwrap()));
            if let None = iter.peek() {
                break;
            }
            cmds.push(arr);
        } else {
            eprintln!("Expected '$', found something else ({:?})", iter.peek());
            return Ok(());
        }
    }
    println!("done.");
    let dev = api.open(VID, PID);
    match dev {
        Err(idk) => {
            eprintln!("oof {}", idk);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                io::Error::last_os_error(),
            ));
        }
        Ok(dev) => {
            println!("Writing");
            for cmd in cmds {
                print!(".");
                dev.write(cmd.as_ref()).unwrap();
                let mut buf = [0u8];
                dev.read_timeout(&mut buf, 100).unwrap();
                if buf[0] != 0x40 {
                    eprintln!("Received {}", buf[0]);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        io::Error::last_os_error(),
                    ));
                }
            }
            println!(" and done!");
        }
    }
    println!("Everything went well!");
    Ok(())
}

fn main() {
    let path = std::env::args().skip(1).next().unwrap();
    println!("Starting...");
    if let Ok(api) = HidApi::new() {
        if let Err(err) = upload_code(path.as_ref(), api) {
            eprintln!("Failed to upload: {}.", err);
        } else {
            println!("Upload successful!");
        }
    } else {
        eprintln!("Failed to open hidapi.")
    }
}
