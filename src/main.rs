use std::env;
use std::process::exit;
use std::io;
use std::io::Write;
use std::fs::*;

use frib_datasource;
//
// Usage:
//    trace-defenestrator in-uri outfile
// See fn usage below.
fn main() {
    let argv : Vec<String> = env::args().collect();
    if argv.len() != 3 {   // Program path is included.
        eprintln!("Incorrect number of command line arguments\n");
        usage();
        exit(-1);
    }

    let in_uri = argv[1].clone();
    let outpath = argv[2].clone();

    // Make the ring source.

    let mut src = 
    match frib_datasource::data_source_factory(&in_uri) {
        Err(e) => {
            eprintln!("Could not create the event source from the URI {}: {}", in_uri, e);
            usage();
            exit(-1);
        },
        Ok(src) => {
            src
        }
    };

    // Open the output file or stdout.:

    let sink : Box<dyn Write> = if outpath == "-" {
        Box::new(io::stdout())
    } else {
        let out = File::create(&outpath);
        match out {
            Err(e) => {
                eprintln!("Could not open the output file {} : {}", outpath, e);
                usage();
                exit(-1);
            },
            Ok(f) =>  {
                Box::new(f)
            }
        }
        
    };

}

fn usage() {
    eprintln!("Usage:");
    eprintln!("   trace-defenestrator in-uri outfile");
    eprintln!("Where");
    eprintln!("   in-uri - is a URI for the data source.");
    eprintln!("            tcp://host/ring is supported for live data");
    eprintln!("   outfile - Is the output file path. Note that");
    eprintln!("           the special value '-' means stdout which,");
    eprintln!("          combined with stdintoring supports contributing to a ringbuffer");

}