use rand::{Rng, RngCore};
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

use std::{time::Instant, u128};

use clap::{arg, value_parser, Command};
use log::{debug, info, trace, Level, LevelFilter, Log, Metadata, Record};

use std::fs;

const MIN_LEN: usize = 0;
const MAX_LEN: usize = 1024;

static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

struct ConsoleLogger;

impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            //println!("{} - {}", record.level(), record.args());
            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}

struct DataFuzzer {
    rng: ChaCha8Rng,
    max_steps: u32,
    max_duration: u128,
    max_len: usize,
    input_files: Vec<String>,
}

impl DataFuzzer {
    fn new() -> Self {
        let rng = ChaCha8Rng::seed_from_u64(1234);
        Self {
            rng,
            max_steps: 0,
            max_duration: 0,
            max_len: MAX_LEN,
            input_files: vec![],
        }
    }

    fn should_stop(&self, step: u32, duration: u128) -> bool {
        if self.max_steps > 0 && step >= self.max_steps {
            return true;
        }

        if self.max_duration > 0 && duration >= self.max_duration {
            return true;
        }
        false
    }

    fn set_input_data(&mut self, path: &String) {
        if fs::metadata(path).unwrap().is_file() {
            self.input_files.push(path.to_string());
        } else {
            let mut files = fs::read_dir(path)
                .unwrap()
                .filter_map(|res| {
                    let dir_entry = res.unwrap();
                    if dir_entry.file_type().unwrap().is_file() {
                        Some(dir_entry.path().to_str().unwrap().to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
            files.sort();
            files.reverse();
            self.input_files = files;
        }
        self.max_steps = self.input_files.len() as u32;
    }

    fn set_max_len(&mut self, len: usize) {
        self.max_len = len;
    }

    fn set_max_steps(&mut self, steps: u32) {
        self.max_steps = steps;
    }

    fn set_duration(&mut self, duration: u128) {
        self.max_duration = duration * 1000;
    }

    fn get_rand_vector(&mut self) -> Vec<u8> {
        let mut len = 0;
        if self.max_len > 0 {
            len = self.rng.gen_range(MIN_LEN..self.max_len);
        }
        let mut vec = vec![0; len];
        if len > 0 {
            self.rng.fill_bytes(vec.as_mut_slice());
        }
        vec
    }

    fn get_data(&mut self) -> Vec<u8> {
        if !self.input_files.is_empty() {
            let file = self.input_files.pop().unwrap();
            trace!("reading file = {}", file);
            fs::read(file).unwrap()
        } else {
            self.get_rand_vector()
        }
    }
}

fn fuzz_init() -> DataFuzzer {
    let matches = Command::new("SimpleFuzzer")
        .about("Simple data fuzzer.

It allows to:
- quickly rebuild when developing new fuzz tests
  (building afl or libfuzzer takes ages...)
- reproduce problematic case (crash, hang) using some input file
  Input file might be generated by afl or libfuzzer when fuzzing.
  It produces more readable stack trace than libfuzzer.")
        .arg(
            arg!(--iterations <VALUE> "Number of iterations")
                .required(false)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            arg!(--duration <VALUE> "Number of seconds to fuzz")
                .required(false)
                .value_parser(value_parser!(u128)),
        )
        .arg(
            arg!(--"max-data-len" <VALUE> "Maximal data len")
                .required(false)
                .value_parser(value_parser!(usize)),
        )
        .arg(
            arg!([input] "Input file (if provided then fuzzer runs once using data from file) or folder with files.\nUseful for reproducing problematic cases.")
                .required(false),
        )
        .arg(arg!(-v --verbose ... "Verbose").required(false))
        .get_matches();

    log::set_logger(&CONSOLE_LOGGER).unwrap();

    let mut fuzzer = DataFuzzer::new();

    if let Some(input) = matches.get_one::<String>("input") {
        fuzzer.set_input_data(input);
    } else {
        if let Some(val) = matches.get_one::<u32>("iterations") {
            fuzzer.set_max_steps(*val);
        }
        if let Some(val) = matches.get_one::<u128>("duration") {
            fuzzer.set_duration(*val);
        }
        if let Some(val) = matches.get_one::<usize>("max-data-len") {
            fuzzer.set_max_len(*val);
        }
    }
    match matches.get_count("verbose") {
        0 => log::set_max_level(LevelFilter::Info),
        1 => log::set_max_level(LevelFilter::Debug),
        _ => log::set_max_level(LevelFilter::Trace),
    }
    fuzzer
}

pub fn fuzz_loop<F>(mut closure: F)
where
    F: FnMut(&[u8]),
{
    let mut fuzzer = fuzz_init();

    let mut cnt = 0_u32;
    let start = Instant::now();
    while !fuzzer.should_stop(cnt, start.elapsed().as_millis()) {
        let data = fuzzer.get_data();
        closure(&data);
        debug!(
            "step= {} duration= {} ms data len={}",
            cnt,
            start.elapsed().as_millis(),
            data.len(),
        );
        cnt += 1;
    }
    info!("Done {} runs in {} s ", cnt, start.elapsed().as_secs());
}

#[macro_export]
macro_rules! fuzz {
    ( $($x:tt)* ) => { $crate::__fuzz!($($x)*) }
}

#[macro_export]
macro_rules! __fuzz {
    (|$buf:ident| $body:block) => {
        $crate::fuzz_loop(|$buf| $body);
    };
    (|$buf:ident: &[u8]| $body:block) => {
        $crate::fuzz_loop(|$buf| $body);
    };
    (|$buf:ident: $dty: ty| $body:block) => {
        $crate::fuzz_loop(|$buf| {
            let $buf: $dty = {
                let mut data = ::arbitrary::Unstructured::new($buf);
                if let Ok(d) = ::arbitrary::Arbitrary::arbitrary(&mut data).map_err(|_| "") {
                    d
                } else {
                    return;
                }
            };

            $body
        });
    };
}
