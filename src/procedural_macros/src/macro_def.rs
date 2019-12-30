#[macro_export]
macro_rules! veca {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

pub fn seat_at_table() {
    println!("Hello seat_at_table");
}
