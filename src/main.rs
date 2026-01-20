use std::env;

//
// Usage:
//    trace-defenestrator in-uri outfile
// See fn usage below.
fn main() {
    usage();
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