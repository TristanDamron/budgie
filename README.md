# Hello Budgie!

Budgie is a *super simple* BDD test framework for Rust. Your Rust code is complicated, your tests shouldn't be!


## Code Sample

In src/math.rs
```
mod math {
    fn add(x: i32, y: i32) -> i32 {
        return x + y;
    }
}
```

In src/test_add.rs:
```
mod math;

use math;
use budgie::*;

describe("Add", || {
    it("correctly adds 2 and 2", || {
        let sum: i32 = math::add(2, 2);
        Assert::<i32>::default().expect(sum).to_be(4);
    });
});
```
