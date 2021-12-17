use std::{env, fs, net::{TcpListener, TcpStream}, io::prelude::*, process::exit};
use exitcode;
use rusty_nail::png::generate_thumbnail;
use clap;


fn get_number_from_stream(socket: &mut TcpStream) -> Result<usize, String> {
    let mut buf: [u8; 32] = [0u8; 32];  // no base-10 size should exceed 32 digits
    let bytes_read: usize;
    match socket.read(&mut buf) {
        Ok(num) => bytes_read = num,
        Err(e) => return Err(format!("failed to read image size from stream: {:?}", e)),
    }
    let number_str = std::str::from_utf8(&buf[..bytes_read]).unwrap().trim();
    match number_str.parse::<usize>() {
        Ok(number) => Ok(number),
        Err(_) => Err(format!("failed to parse data as number `{}` as usize", number_str)),
    }
}


/// Expects to receive two TCP connections. The first should be a single line
/// which states the size of the png file, such as what is produced by
///     `du -b <filename> | awk -F ' ' '{print $1}'`
/// The second should be the data from that file, and the resulting thumbnail
/// is sent back as a response to the second connection.
///
/// Recommended to use tools/send_image_over_tcp.sh
fn handle_thumbnail_over_tcp(width: usize, height: usize, crop: bool,
                             address: &str) -> Result<(), String> {
    let tcp_listener: TcpListener;
    match TcpListener::bind(address) {
        Ok(listener) => tcp_listener = listener,
        Err(e) => return Err(format!("failed to bind to address: {}: {:?}", address, e)),
    };
    let png_data_length: usize;
    match tcp_listener.accept() {
        Ok((mut socket, _addr)) => match get_number_from_stream(&mut socket) {
            Ok(number) => png_data_length = number,
            Err(e) => return Err(e),
        },
        Err(e) => return Err(format!("first connection to client failed: {:?}", e)),
    }
    let mut image_data: Vec<u8> = vec![0u8; png_data_length];
    let mut second_socket: TcpStream;
    match tcp_listener.accept() {
        Ok((mut socket, _addr)) => {
            socket.read_exact(image_data.as_mut_slice()).unwrap();
            second_socket = socket;
        },
        Err(e) => return Err(format!("second connection to client failed: {:?}", e)),
    }
    let thumbnail_data: Vec<u8>;
    match generate_thumbnail(image_data, width, height, crop) {
        Ok(data) => thumbnail_data = data,
        Err(e) => return Err(format!("failed to generate thumbnail: {:?}", e)),
    }
    second_socket.write_all(thumbnail_data.as_slice()).unwrap();
    match second_socket.flush() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("failed to flush data written to tcp stream: {:?}", e)),
    }
}


fn handle_thumbnail_over_fs(width: usize, height: usize, crop: bool,
                            infile: &str, outfile: &str) -> Result<(), String> {
    let image_data: Vec<u8>;
    match fs::read(&infile) {
        Ok(data) => image_data = data,
        Err(e) => return Err(format!("failed to read file `{}`: {}", &infile, e)),
    }
    let thumbnail_data: Vec<u8>;
    match generate_thumbnail(image_data, width, height, crop) {
        Ok(data) => thumbnail_data = data,
        Err(e) => return Err(format!("failed to generate thumbnail: {:?}", e)),
    }
    match fs::write(&outfile, &thumbnail_data) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("failed to write thumbnail to file `{}`: {}", &outfile, e)),
    }
}


fn parse_dimensions(matches: &clap::ArgMatches, default_width: usize,
                    ) -> Result<(usize,usize,bool), String> {
    let width: usize = match matches.value_of("width") {
        Some(number) => {
            let val: usize;
            match number.parse::<usize>() {
                Ok(size) => val = size,
                Err(_) => return Err(format!("failed to parse width `{}` as usize", number)),
            }
            val
        },
        None => default_width,
    };
    let height: usize = match matches.value_of("height") {
        Some(number) => {
            let val: usize;
            match number.parse::<usize>() {
                Ok(size) => val = size,
                Err(_) => return Err(format!("failed to parse height `{}` as usize", number)),
            }
            val
        },
        None => width,
    };
    let crop: bool = matches.is_present("crop");
    Ok((width, height, crop))
}


fn parse_args<'a>() -> clap::ArgMatches<'a> {
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(clap::Arg::with_name("image")
             .short("i")
             .long("image")
             .takes_value(true)
             .value_name("/PATH/TO/IMAGE")
             .help("the path to the original image")
             .requires("thumbnail"))
        .arg(clap::Arg::with_name("thumbnail")
             .short("t")
             .long("thumbnail")
             .takes_value(true)
             .value_name("/PATH/TO/THUMBNAIL")
             .help("the path to the thumbnail image")
             .requires("image"))
        .group(clap::ArgGroup::with_name("paths")
               .args(&["image", "thumbnail"])
               .multiple(true))
        .arg(clap::Arg::with_name("address")
             .short("a")
             .long("address")
             .takes_value(true)
             .value_name("ADDR:PORT")
             .help("the ip address:port to read/write data (of the form 'localhost:12345')"))
        .group(clap::ArgGroup::with_name("inputs_conflict")
               .args(&["paths", "address"])
               .multiple(false))
        .group(clap::ArgGroup::with_name("inputs_require")
               .args(&["image", "thumbnail", "address"])
               .required(true)
               .multiple(true))
        .arg(clap::Arg::with_name("width")
             .short("x")
             .long("width")
             .takes_value(true)
             .value_name("WIDTH")
             .help("the width of the thumbnail (defaults to 150)"))
        .arg(clap::Arg::with_name("height")
             .short("y")
             .long("height")
             .takes_value(true)
             .value_name("HEIGHT")
             .help("the height of the thumbnail (defaults to match width)"))
        .arg(clap::Arg::with_name("crop")
             .short("c")
             .long("crop")
             .takes_value(false)
             .help("crop the image to exactly fill the given thumbnail dimensions"))
        .get_matches();
    matches
}


fn main() {
    let matches = parse_args();
    let default_width = 150;
    let (width, height, crop) = parse_dimensions(&matches, default_width)
        .unwrap_or_else(|err| {
            eprintln!("ERROR: {}", err);
            eprintln!("{}", matches.usage());
            exit(exitcode::USAGE);
        });
    match if matches.is_present("address") {
        handle_thumbnail_over_tcp(width, height, crop,
                                  matches.value_of("address").unwrap())
    } else {
        handle_thumbnail_over_fs(width, height, crop,
                                 matches.value_of("image").unwrap(),
                                 matches.value_of("thumbnail").unwrap())
    } {
        Ok(_) => (),
        Err(message) => {
            eprintln!("ERROR: {}", message);
            exit(exitcode::DATAERR);
        },
    }
}
