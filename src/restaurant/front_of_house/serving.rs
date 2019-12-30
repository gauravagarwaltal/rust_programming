fn take_order() {
    println!("Hello take_order");
}

pub fn serve_order() {
    take_order();
    println!("Hello serve_order");
    take_payment();
}

fn take_payment() {
    println!("Hello take_payment");
}
