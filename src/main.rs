use budgie::*;

fn main() {
    monitor();

    describe("Testing math functions", || {
        after_each(|| println!(""));

        before_each(|| println!(""));

        it("adding 2 and 2 should equal 4", || {
            let sum: BudgieValue<i32> = BudgieValue {
                true_val: 2 + 2,
                fmt_val: format!("2 + 2"),
            };

            Assert::<i32> {
                compare_val: sum,
                expect: true,
            }
            .to_be_fmt(4, "four");
        });

        it("subtracting 2 and 2 should equal 0", || {
            let difference: i32 = 2 - 2;

            Assert::<i32>::default()
                .expect_fmt(difference, "2 - 2")  // 0
                .to_be_fmt(0, "zero");
        });

        it("dividing 1 and 2 should be close to 1 by a half", || {  // TODO: implement to_be_close_to
            let divided: f32 = 1. / 2.;

            PartialAssert::<f32>::default()
                .expect_fmt(divided, "1 / 2")
                .to_be_fmt(0.5, "one half");
        });

        it("add 3 and 2 should equal 5", || {
            Assert::<i32>::default()
                .expect(3 + 2)
                .to_be_fmt(5, "five");
        });
    });
}
