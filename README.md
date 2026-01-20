# trace-defenestrator
Defenestrate trace (analog data):

Frame data is used in streaming daq systems.  Frames are fixed
sized time chunks. 

Analog frame data consists of FRIB/NSCLDAQ ring items of type 50.
The ring item has a body header and the timsestamp of the body header
is the timestamp at start of frame.  The source id and barrier id are
as normal.   

The body of the ring item contains:

| Item       | size  | Contains                                         |
|------------|-------|--------------------------------------------------|
| Chunk-size | u32   | amount of non-zero data in the frame  in samples |
| Offset     | u16   | Offset in samples in the frame where data starts |
| data       | []u16 | Chunk-size u16 items that are the data           |

Note that a waveform _can_ and often will span frame boundaries.  If,
a frame's offset and chunk-size run up to the end of a frame and the next frame has offset 0 and chunk-size non-zero, the next frame is considered
to have a continuation of the waveform in the previous frame.

Frame size is assumed to be chosen such that a frame will have at most
one waveform.

Still to resolve - the sampling frequency needs to be known...and for the
test data there is some timing nastiness in that the timestamp of the original data are nanoseconds so the frames are some number of ns long but the samples are whatever frequency the digitizer was runningt which is significantly slower than a ns/sample.  Frame data probably _should_ use a raw timestamp so that frames are in raw timestamp units rather than the cooked ns timestamp...but this at least lets us play...