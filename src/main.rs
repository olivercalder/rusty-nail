use std::{env, fs, process::exit};
use exitcode;
use rusty_nail::png::generate_thumbnail;


const USAGE: &str = "
USAGE: rusty-nail FILENAME OUTFILENAME [WIDTH] [HEIGHT] [FILL_SIZE]
\nFILENAME\tthe original image filename
OUTFILENAME\tthe thumbnail output filename
WIDTH\t\twidth of thumbnail -- defaults to 150
HEIGHT\t\theight of thumbnail -- defaults to WIDTH
FILL_SIZE\tcrop the image to exactly fill the given thumbnail dimensions
";


struct Params {
    infile: String,
    outfile: String,
    width: usize,
    height: usize,
    fill: bool,
}


fn parse_args(args: &[String], default_width: usize, default_height: usize,
              default_fill: bool) -> Result<Params, String> {
    let mut width: usize = default_width;
    let mut height: usize = default_height;
    let mut fill: bool = default_fill;
    if args.len() < 2 {
        return Err("missing required argument: FILENAME".to_string());
    }
    if args.len() < 3 {
        return Err("missing required argument: OUTFILENAME".to_string());
    }
    let infile: String = (&args[1]).to_string();
    let outfile: String = (&args[2]).to_string();
    if args.len() >= 4 {
        match args[3].parse::<usize>() {
            Ok(size) => width = size,
            Err(_) => return Err(format!("failed to parse `{}` as usize", args[3])),
        }
    }
    if args.len() >= 5 {
        match args[4].parse::<usize>() {
            Ok(size) => height = size,
            Err(_) => return Err(format!("failed to parse `{}` as usize", args[4])),
        }
    }
    if args.len() == 6 {
        fill = true;
    }
    Ok(Params { infile, outfile, width, height, fill, })
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let default_width = 150;
    let default_height = 150;
    let default_fill = false;
    let params: Params = parse_args(&args, default_width, default_height,
                                    default_fill).unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        eprintln!("{}", USAGE);
        exit(exitcode::USAGE);
    });
    let orig_data: Vec<u8>;
    match fs::read(&params.infile) {
        Ok(data) => orig_data = data,
        Err(e) => {
            eprintln!("ERROR: {}: {}", e, &params.infile);
            eprintln!("{}", USAGE);
            exit(exitcode::NOINPUT)
        },
    }
    let thumbnail_data: Vec<u8>;
    match generate_thumbnail(orig_data, params.width, params.height, params.fill) {
        Ok(data) => thumbnail_data = data,
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            exit(exitcode::DATAERR)
        },
    }
    match fs::write(&params.outfile, &thumbnail_data) {
        Ok(_) => exit(exitcode::OK),
        Err(e) => {
            eprintln!("ERROR: {}: {}", e, &params.outfile);
            exit(exitcode::CANTCREAT)
        },
    }
}
