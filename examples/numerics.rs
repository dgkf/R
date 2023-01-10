use r::r_vector::vectors::*;

fn main() {
    let x = RVector::from(vec![false, true, false]);
    let y = RVector::from(vec![1_i32]);

    println!("Example 1:\nRecycled add of Vec<bool> and Vec<i32>");
    println!("{} + {}", x, y);
    println!("{}\n", x + y);

    use r::r_vector::vectors::OptionNA::*; // Some() and NA
    let x = RVector::from(vec![Some(5_i32), NA, Some(1_i32)]);
    let y = RVector::from(vec![1_f64, 2_f64, 3_f64]);

    println!("Example 2:\nAdd of Vec<i32/NA> and Vec<f64>");
    println!("{} + {}", x, y);
    println!("{}\n", x + y);
}
