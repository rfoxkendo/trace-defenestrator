use std::env;
use std::process::exit;
use std::io;
use std::io::Write;
use std::fs::*;

use frib_datasource;
use frib_datasource::{DataSource, DataSink};
use rust_ringitem_format::{RingItem, BodyHeader, ToRaw};
use rust_ringitem_format::event_item::*;   // For PHYSICS_EVENT.


const TRACE_FRAME_ITEM_TYPE : u32 = 50;
const FRAME_LENGTH : u64          = 512;   // Ticks in a frame.
// This is what a trace looks like:
#[derive(Debug, Clone)]
struct Trace {
    timestamp: u64,        // Fine timestamp.
    waveform: Vec<u16>,    // The trace data.
}
impl Trace {
    pub fn new(ts : u64) -> Trace {
        Trace {
            timestamp : ts,
            waveform : Vec::new()
        }
    }
}
#[derive(Debug)]
struct Frame {
    frame_start: u64,        // Coarse timestamp - frame start.
    data_size: u32,          // data size samples in the frame.
    data_offset: u16,        // where in the frame the samples start.
    data: Vec<u16>,          // data_size samples.
}

impl Frame {
    pub fn new(start : u64) -> Frame {
        Frame {
            frame_start : start,
            data_size   : 0,        // Must be computed
            data_offset : 0,        // Must be computed.
            data : Vec::new(),
        }
    }
}
//
// Usage:
//    trace-defenestrator in-uri out-uri
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

    let sink = frib_datasource::data_sink_factory(&outpath);
    if let Err(e) = sink {
        eprintln!("Failed to open the data sink {} : {}", outpath, e);
        usage();
        exit(-1);
    };
    let mut sink = sink.unwrap();
    // Process the inputs to the outputs.

    frames_to_traces(&mut src, &mut sink);
}

fn usage() {
    eprintln!("Usage:");
    eprintln!("   trace-defenestrator in-uri outfile");
    eprintln!("Where");
    eprintln!("   in-uri - is a URI for the data source.");
    eprintln!("            tcp://host/ring is supported for live data");
    eprintln!("   out-uri - Is the output file path. Note that");
    eprintln!("           the special value 'file:///-' means stdout which,");
    eprintln!("           out-uri functions exactly like in-uri but only localhost ");
    eprintln!("           can be used as the host.");

}
// This function has the main logic...accepting ring items with frames from the source
// and emitting ring items with traces on the sink

fn frames_to_traces(src: &mut  Box<dyn DataSource>, sink : &mut Box<dyn DataSink>) {
    let mut trace : Option<Trace> = None;             // Some if we are assembling.
 
    while let Some(item) = src.read() {
        // Hunt for a frame with data:
        let frame : Frame = item_to_frame(&item);
        
        
        // trace holds the state, do what's needed given the state and the data

        if trace.is_some() {
            let mut t = trace.unwrap();
            if frame.data_size == 0 {
                // Empty frame means we're done with the trace:

                write_trace(sink, &t);
                trace = None;                            // Not assembling.
            } else {
                if frame.data_offset == 0  {            // assemble into current frame.
                    t.waveform.extend(&frame.data[..]);

                    // If the data extends to the end of the frame, there could be more, else
                    // done.

                    if frame.data.len() == FRAME_LENGTH as usize {
                        trace = Some(t);               // ready for more.
                    } else {                           // complete trace
                        write_trace(sink, &t);
                        trace  = None;                 // no longer assembling.

                    }
                } else {
                    // We have a trace but this isn't an extension to it so flush and start a new trace.

                    write_trace(sink, &t);
                    
                    // Make the new trace:

                    let new_trace = start_trace(&frame);
                    trace = Some(new_trace);
                }
            }
        } else { 
            // There's no trace yet .. to make one requires a frame with some data:

            if frame.data_size > 0 {
                let new_trace = start_trace(&frame);
                trace = Some(new_trace);
            }
            // empty frame so nothing to do.
        }

    }
    // If at the end of all of this, I still have a a trace I need to dump it:

    if let Some(t) = trace {
        write_trace(sink, &t);
    }
}
// start a new trace from a frame.
fn start_trace(f : &Frame) -> Trace {
    let mut t = Trace::new(f.frame_start + f.data_offset as u64);
    t.waveform.extend(&f.data[..]);
    t
}

// Convert a ring item to a frame or panic if the item is the wrong type:
// For now we don't worry about source-ids.
//
fn item_to_frame(item : &RingItem) -> Frame {
    if item.type_id() != TRACE_FRAME_ITEM_TYPE {
        panic!("Ring item was not a frame should be type {} was {}", TRACE_FRAME_ITEM_TYPE, item.type_id());
    }
    if !item.has_body_header() {
        panic!("Trace ring item does not have a body header from which to extract the timestamp!");
    }
    let body_header = item.get_bodyheader().unwrap();
    let mut result = Frame::new(body_header.timestamp);

    // Get the frame description fromt he ring item:
    // Note that the payload includes the body header contents!

    let body = item.payload();    // Vec<u8>
    let body = &body[4*size_of::<u32>()..];  // Skip over the body header we know we have.
    let longbuf  = &body[0..4];
    result.data_size = u32::from_ne_bytes(longbuf.try_into().unwrap());

    let shortbuf  = &body[4..6];
    result.data_offset = u16::from_ne_bytes(shortbuf.try_into().unwrap());

    // If there's data, we need to stuff it in:

    if result.data_size > 0 {
        let mut i = 6;
        for  _ in 0..result.data_size {
            let shortbuf  = &body [i..i+size_of::<u16>()];

            let sample = u16::from_ne_bytes(shortbuf.try_into().unwrap());
            result.data.push(sample);
            i += size_of::<u16>();
        }
    }
    result

}

// Write a trace as a PHYSICS_EVENT ring item to a sink.

fn write_trace(sink : &mut Box<dyn DataSink>, trace : &Trace) {
    let body_header = BodyHeader {
        timestamp: trace.timestamp,
        source_id: 0,                               // For now.
        barrier_type: 0
    };

    let mut event = PhysicsEvent::new(Some(body_header));  // Physics item with a body header as above.

    // put the waveform is as the body:

    for sample in &trace.waveform {
        event.add(*sample);
    }

    // Write it out... for now panic on failure.

    let item = event.to_raw();
    sink.write(&item).expect("Unable to write item to data sink");

    // for now flush every time:

    sink.flush();
    
}