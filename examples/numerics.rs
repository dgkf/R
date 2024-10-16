use r::object::Vector;

fn main() {
    let x = Vector::from(vec![false, true, false]);
    let y = Vector::from(vec![1_i32]);

    println!("Example 1:\nRecycled add of Vec<bool> and Vec<i32>");
    println!("{} + {}", x, y);
    println!("{}\n", (x + y).unwrap());

    use r::object::OptionNA::*; // Some() and NA
    let x = Vector::from(vec![Some(5_i32), NA, Some(1_i32)]);
    let y = Vector::from(vec![1_f64, 2_f64, 3_f64]);

    println!("Example 2:\nAdd of Vec<i32/NA> and Vec<f64>");
    println!("{} + {}", x, y);
    println!("{}\n", (x + y).unwrap());
}
