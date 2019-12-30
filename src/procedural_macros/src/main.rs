// pub mod procedural_macros;
#[macro_use] mod macro_def;

macro_rules! foo {
    ($v:ident) => (let $v = 3;);
}


fn main(){
    macro_def::seat_at_table();

    let x : Vec<i32> = veca![1, 2, 3, 4, 3, 4];
    println!("{:?}", x);
    foo!(ident);
    println!("{}", ident);
}