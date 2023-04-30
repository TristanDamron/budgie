extern crate textplots;

use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::thread;
use std::time::SystemTime;
use json::JsonValue;
use textplots::{Chart, Plot, Shape};
use json::object;

pub static mut MONITOR: bool = false;
pub static mut BEFORE_EACH: fn() = || {};
pub static mut AFTER_EACH: fn() = || {};
pub static mut TEST_COUNTER: u16 = 0;
pub static mut TIME_STAMP: u128 = 0;


pub fn describe(msg: &str, func: fn()) -> () {
    let start_duration: SystemTime = SystemTime::now();
    let mut process_end_time: f32 = 0.;

    println!("🦜 Running Test: {}", msg);
    func();

    unsafe {
        if MONITOR {
            match SystemTime::now().duration_since(start_duration) {
                Ok(n) => process_end_time = n.as_secs_f32(),
                Err(_) => panic!("Failed to get epoch from system"),
            }

            let file: File = OpenOptions::new()
                .read(true)
                .open(format!(".budgie/{}.log", TIME_STAMP))
                .unwrap();

            let reader: BufReader<File> = BufReader::new(file);
            let mut performance_chart: Chart = Chart::new(200, 100, 0., f32::from(TEST_COUNTER));
            let mut vec_points: Vec<(f32, f32)> = Vec::new();
            let mut vec_tests: Vec<(String, f32)> = Vec::new();
            let mut counter: f32 = 0.;

            for line in reader.lines() {
                let str_line: String = line.unwrap();
                let parsed_json: JsonValue = json::parse(&str_line).unwrap();
                let thread_end_time = parsed_json["thread_end_time"].as_f32().unwrap();
                vec_points.push((counter, thread_end_time));
                vec_tests.push((parsed_json["test_name"].as_str().unwrap().to_string(), thread_end_time));
                counter += 1.;
            }

            vec_tests.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            vec_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            vec_tests.reverse();
            vec_points.reverse();

            let mut i: usize = 0;
            counter = 0.;
            while i < vec_points.len() {  // Re-number tests from 0 -> vec_points.len()
                vec_points[i] = (f32::from(counter), vec_points[i].1);
                counter += 1.;
                i += 1;
            }
            
            println!("{} Performance Chart {}", "=".repeat(10), "=".repeat(10));
            performance_chart
                .lineplot(&Shape::Bars(vec_points.as_slice()))
                .display();
            
            println!("{} Tests Ran {}", "=".repeat(10), "=".repeat(10));
            i = 0;
            while (i < vec_tests.len()) {
                println!("{}: {}s", vec_tests[i].0, vec_tests[i].1);
                i += 1;
            }

            println!("{}", "=".repeat(20));
            println!("Ran {} tests in {} seconds", TEST_COUNTER, process_end_time);
        }
    }
}

pub fn it(msg: &str, func: fn()) -> () {
    println!("{} {} {}", "=".repeat(10), msg, "=".repeat(10));

    let base_thread_json_data: JsonValue = object!{
        thread_end_time: 0,
        test_name: format!("{}", msg)
    };

    unsafe {
        if MONITOR {
            if TIME_STAMP == 0 {
                TIME_STAMP = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
            }

            create_dir_all("./.budgie/").unwrap();
            
            let file_name: String = format!(".budgie/{}.log", TIME_STAMP);
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(file_name)
                .unwrap();

            thread::spawn(move || {
                let thread_start_duration: SystemTime = SystemTime::now();

                BEFORE_EACH();
                TEST_COUNTER += 1;

                func();

                AFTER_EACH();

                let mut cpy_json_data: JsonValue = base_thread_json_data.clone();

                cpy_json_data["thread_end_time"] = SystemTime::now().duration_since(thread_start_duration).unwrap().as_secs_f32().into();

                match writeln!(file, "{}", cpy_json_data.dump()) {
                    Err(why) => panic!("Failed to write line to file: {}", why),
                    Ok(_) => {}
                }

            })
            .join()
            .unwrap();
        } else {
            thread::spawn(move || {
                BEFORE_EACH();
                TEST_COUNTER += 1;

                func();

                AFTER_EACH();
            })
            .join()
            .unwrap();
        }
    }
}

pub fn before_each(func: fn()) {
    unsafe {
        BEFORE_EACH = func;
    }
}

pub fn after_each(func: fn()) {
    unsafe {
        AFTER_EACH = func;
    }
}

pub fn monitor() {
    unsafe {
        MONITOR = true;
    }
}

#[derive(Default)]
pub struct BudgieValue<T> {
    pub true_val: T,
    pub fmt_val: String,
}

#[derive(Default)]
pub struct Assert<T: PartialOrd + PartialEq> {
    pub compare_val: BudgieValue<T>,
    pub expect: bool,
}

impl<T: PartialOrd + PartialEq> Assert<T> {
    pub fn expect(mut self, val: T) -> Self {
        if !self.expect {
            self.compare_val.true_val = val;
            self.expect = true;
            return self;
        } else {
            panic!("expect() must not be chained to any other functions.");
        }
    }

    pub fn expect_fmt(mut self, val: T, fmt_val: &str) -> Self {
        self.compare_val.fmt_val = fmt_val.to_string();
        return self.expect(val);
    }

    fn assert_to_be(self, val: BudgieValue<T>) -> Self {
        if self.expect {
            if val.true_val == self.compare_val.true_val {
                if val.fmt_val.is_empty() || self.compare_val.fmt_val.is_empty() {
                    println!("✅ PASSED");
                } else {
                    println!(
                        "✅ PASSED: {} is equal to {}",
                        self.compare_val.fmt_val, val.fmt_val
                    );
                }
            } else {
                if val.fmt_val.is_empty() || self.compare_val.fmt_val.is_empty() {
                    println!("🛑 FAILED");
                } else {
                    println!(
                        "🛑 FAILED: {} is NOT equal to {}",
                        self.compare_val.fmt_val, val.fmt_val
                    );
                }
            }
        } else {
            panic!("You must chain to_be() to an expect() function call.");
        }
        return self;
    }

    pub fn to_be(self, val: T) -> Self {
        let val: BudgieValue<T> = BudgieValue {
            true_val: val,
            fmt_val: format!(""),
        };

        return self.assert_to_be(val);
    }

    pub fn to_be_fmt(self, val: T, fmt_val: &str) -> Self {
        let val: BudgieValue<T> = BudgieValue {
            true_val: val,
            fmt_val: fmt_val.to_string(),
        };

        return self.assert_to_be(val);
    }
}
