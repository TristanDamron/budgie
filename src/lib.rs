extern crate textplots;

use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::fs::{remove_file, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::thread;
use std::time::SystemTime;
use textplots::{Chart, Plot, Shape};

pub static mut MONITOR: bool = false;
pub static mut BEFORE_EACH: fn() = || {};
pub static mut AFTER_EACH: fn() = || {};
pub static mut TEST_COUNTER: u16 = 0;

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
                .open(".budgie_performance.log")
                .unwrap();

            let reader: BufReader<File> = BufReader::new(file);
            let mut performance_chart: Chart = Chart::new(200, 100, 0., f32::from(TEST_COUNTER));
            let mut vec_points: Vec<(f32, f32)> = Vec::new();
            let mut line_counter: f32 = 0.;

            for line in reader.lines() {
                let str_line: String = line.unwrap();
                let parsed_runtime: f32 = str_line.parse().unwrap();
                vec_points.push((line_counter, parsed_runtime));
                line_counter += 1.;
            }

            performance_chart
                .lineplot(&Shape::Bars(vec_points.as_slice()))
                .display();

            match remove_file(Path::new(".budgie_performance.log")) {
                Err(why) => panic!("Failed to delete performance log: {}", why),
                Ok(_) => {}
            }

            println!("Ran {} tests in {} seconds", TEST_COUNTER, process_end_time);
        }
    }
}

pub fn it(msg: &str, func: fn()) -> () {
    println!("{} {} {}", "=".repeat(10), msg, "=".repeat(10));
    let mut file: File = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(".budgie_performance.log")
        .unwrap();

    thread::spawn(move || {
        let mut thread_end_time: f32 = 0.;
        let mut thread_start_duration: SystemTime = SystemTime::now();

        unsafe {
            BEFORE_EACH();
            TEST_COUNTER += 1;
        }

        func();

        unsafe {
            AFTER_EACH();
            if MONITOR {
                match SystemTime::now().duration_since(thread_start_duration) {
                    Ok(n) => thread_end_time = n.as_secs_f32(),
                    Err(_) => panic!("Failed to get epoch from system"),
                }

                match writeln!(file, "{}", thread_end_time) {
                    Err(why) => panic!("Failed to write line to file: {}", why),
                    Ok(_) => {}
                }
            }
        }
    })
    .join()
    .unwrap();
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
pub struct Assert<T: Ord + Eq> {
    pub compare_val: BudgieValue<T>,
    pub expect: bool,
}

impl<T: Ord + Eq> Assert<T> {
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

#[derive(Default)]
pub struct PartialAssert<T: PartialOrd + PartialEq> {
    pub compare_val: BudgieValue<T>,
    pub expect: bool,
}

impl<T: PartialOrd + PartialEq> PartialAssert<T> {
    pub fn expect(mut self, val: T) -> Self {
        if !self.expect {
            self.compare_val.true_val = val;
            self.compare_val.fmt_val = "Expected value".to_string();
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

    pub fn fmt(mut self, fmt_val: String) -> Self {
        self.compare_val.fmt_val = fmt_val;
        return self;
    }

    fn partial_assert_to_be(self, val: BudgieValue<T>) -> Self {
        if self.expect {
            if val.true_val == self.compare_val.true_val {
                if val.fmt_val.is_empty() || self.compare_val.fmt_val.is_empty() {
                    println!(
                        "✅ PASSED: {} is equal to {}",
                        self.compare_val.fmt_val, val.fmt_val
                    );
                } else {
                    println!("✅ PASSED");
                }
            } else {
                if val.fmt_val.is_empty() || self.compare_val.fmt_val.is_empty() {
                    println!(
                        "🛑 FAILED: {} is NOT equal to {}",
                        self.compare_val.fmt_val, val.fmt_val
                    );
                } else {
                    println!("🛑 FAILED");
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

        return self.partial_assert_to_be(val);
    }

    pub fn to_be_fmt(self, val: T, fmt_val: &str) -> Self {
        let val: BudgieValue<T> = BudgieValue {
            true_val: val,
            fmt_val: fmt_val.to_string(),
        };

        return self.partial_assert_to_be(val);
    }
}
