use std::time::Instant;
use std::fmt;

#[derive(Debug)]
pub struct TransferStats {
    elapsed: f32,
    bytes: usize,
    instant: Instant 
}

impl fmt::Display for TransferStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //Use Kb/s and stuff
        let bits = self.bytes * 8;

        let (rate, rate_name) = 
        if bits > 1000000000 {
            (bits as f32 / 1000000000.0, "Gb")
        }
        else if bits > 1000000 {
            (bits as f32 / 1000000.0, "Mb")
        }
        else if bits > 1000 {
            (bits as f32 / 1000.0, "Kb")
        }
        else {
            (bits as f32, "b")
        };

        write!(f, "{} {}/s", rate, rate_name)
    }
}

impl TransferStats {
    pub fn new() -> TransferStats {
        TransferStats { elapsed: 0.0, bytes: 0, instant: Instant::now() }
    }

    //pub fn start(&mut self) {
    //    self.instant = Instant::now();
    //}

    pub fn stop(&mut self, bytes: usize) {
        self.elapsed = self.instant.elapsed().as_nanos() as f32 / 1000.0;
        self.bytes = bytes;
    }
}

#[derive(Debug)]
pub struct RealtimeStats {
    instant: Instant,
    current_bytes: usize,
    size: usize,
    measures: Vec<(u64, usize)>
}

impl RealtimeStats {
    pub fn new() -> RealtimeStats {
        RealtimeStats { instant: Instant::now(), current_bytes: 0, size: 0, measures: vec![(0, 0)] }
    }

    pub fn set_size(&mut self, size: usize) {
        if self.size == 0 {
            self.size = size
        }
    }
    pub fn add_bytes(&mut self, bytes: usize) {
        self.measures.push((self.instant.elapsed().as_nanos() as u64, self.current_bytes));
        self.current_bytes += bytes;
    }

    // Return speed in bits
    pub fn _get_speed(&mut self) -> (usize, u64) {
        // Look at past second
        const _SECOND: u64 = 1000000000;

        let (last_time, last_bytes) = match self.measures.last() {
           Some((last_time, last_bytes)) => (*last_time, *last_bytes),
           None => (0, 0)
        }; 

        if (last_time, last_bytes) != (0, 0) {
            let threshold = if last_time > _SECOND { last_time - _SECOND } else { 0 };

            let mut c = 0;
            for m in self.measures.iter() {
                if m.0 < threshold {
                    c += 1;
                }
                else {
                    break;
                }
            }

            for _i in 0..c {
                self.measures.remove(0);
            }

            if let Some((start_time, start_bytes)) = self.measures.first() {
                (last_bytes - start_bytes, last_time - start_time)
            }
            else {
               (self.current_bytes - last_bytes, _SECOND)
            }

        }
        else {
            println!("nope");
           (0, _SECOND)
        }
    }
}
