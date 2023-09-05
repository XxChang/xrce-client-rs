const BEST_EFFORT_STREAM_THRESHOLD: u8 = 1;
const RELIABLE_STREAM_THRESHOLD: u8 = 128;

pub enum StreamType
{
    NoneStream,
    BestEffortStream,
    ReliableStream,
    SharedMemoryStream,
}

pub enum StreamDirection
{
    InputStream,
    OutputStream,
}

pub struct StreamId
{
    pub raw: u8,
    pub index: u8,
    pub type_u: StreamType,
    pub direction: StreamDirection,
}

impl StreamId {
    fn new(index: u8, type_u: StreamType, direction: StreamDirection) -> Self
    {
        StreamId {
            raw: match type_u 
                {
                    StreamType::NoneStream => { 0 },
                    StreamType::BestEffortStream => { index + BEST_EFFORT_STREAM_THRESHOLD },
                    StreamType::ReliableStream => { index + RELIABLE_STREAM_THRESHOLD },
                    StreamType::SharedMemoryStream => { 0 },
                },
            index,
            type_u,
            direction, 
        }
    }

    pub fn from_raw(stream_id_raw: u8, direction: StreamDirection) -> Self
    {
        let (index, type_u) = 
        if BEST_EFFORT_STREAM_THRESHOLD > stream_id_raw
        {
            (
                stream_id_raw,
                StreamType::NoneStream,
            )
        }
        else if RELIABLE_STREAM_THRESHOLD > stream_id_raw
        {
            (
                stream_id_raw - BEST_EFFORT_STREAM_THRESHOLD,
                StreamType::BestEffortStream,
            )
        }
        else
        {
           (
                stream_id_raw - RELIABLE_STREAM_THRESHOLD,
                StreamType::ReliableStream,
           ) 
        };

        StreamId { raw: stream_id_raw, index, type_u, direction }
    }
}

